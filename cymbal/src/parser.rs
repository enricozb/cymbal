use std::{collections::HashSet, path::Path};

use anyhow::{Context, Result};
use futures::{Stream, StreamExt};
use tree_sitter::{Parser as TreeSitterParser, Point, QueryCursor};

use crate::{
  config::{Config, Language, Queries},
  ext::{IntoStream, PathExt, ResultExt, StrExt, TreeSitterParserExt},
  symbol::Symbol,
  utils::Lazy,
};

pub struct Parser<'a> {
  path: &'a Path,
  language: Language,
  queries: Option<&'static Lazy<Queries>>,
}

impl<'a> Parser<'a> {
  pub fn new(path: &'a Path, language: Language, config: &'static Config) -> Self {
    let queries = config.queries_for_language(language);

    Self { path, language, queries }
  }

  pub async fn symbol_stream(self) -> Result<impl Stream<Item = Symbol>> {
    let language = self.language;
    let mut parser = TreeSitterParser::with_language(self.language)?;
    let content_bytes = self.path.read_bytes().await?;
    let tree = parser.parse(&content_bytes, None).context("failed to create parser")?;

    self
      .queries
      .into_iter()
      .flat_map(|queries| queries.iter())
      .stream()
      .flat_map(|(kind, queries)| queries.stream().map(move |query| (kind, query)))
      .filter_map(|(kind, query)| async move {
        query
          .tree_sitter_query()
          .capture_index_for_name("symbol")
          .map(|symbol_index| (kind, query, symbol_index))
      })
      .flat_map(move |(kind, query, symbol_index)| {
        let mut cursor = QueryCursor::new();
        let mut start_points = HashSet::new();
        let mut matches = cursor.matches(query.tree_sitter_query(), tree.root_node(), content_bytes.as_slice());
        let mut symbols = Vec::new();

        while let Some(m) = tree_sitter::StreamingIterator::next(&mut matches) {
          let Some(capture) = m.captures.iter().find(|q| q.index == symbol_index) else { continue };

          let node = capture.node;
          let start_point @ Point { row, column } = node.start_position();

          if start_points.contains(&start_point) {
            continue;
          }
          start_points.insert(start_point);

          let start_byte = node.start_byte();
          let end_byte = node.end_byte();
          let symbol_content_bytes = &content_bytes[start_byte..end_byte];
          let Some(symbol_content_str) = symbol_content_bytes.to_str() else { continue };

          let leading = query.leading().map(|t| t.render(m, content_bytes.as_slice())).and_then(Result::ok);
          let trailing = query.trailing().map(|t| t.render(m, content_bytes.as_slice())).and_then(Result::ok);

          symbols.push(Symbol {
            kind: *kind,
            language,
            line: row as u64 + 1,
            column: column as u64,
            content: symbol_content_str.to_string(),
            leading,
            trailing,
          });
        }

        symbols.stream()
      })
      .ok()
  }
}
