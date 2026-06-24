use std::path::Path;

use anyhow::{Context, Result};
use futures::Stream;
use tree_sitter::{Parser as TreeSitterParser, Point, QueryCursor, StreamingIterator};

use crate::{
  config::{Config, Language, LanguageQuery},
  ext::{IntoExt, IteratorExt, PathExt, StrExt, TreeSitterParserExt},
  symbol::Symbol,
  utils::Lazy,
};

pub struct Parser<'a> {
  file_path: &'a Path,
  language: Language,
  queries: Option<&'static Lazy<LanguageQuery>>,
}

impl<'a> Parser<'a> {
  pub fn new(file_path: &'a Path, language: Language, config: &'static Config) -> Self {
    let queries = config.queries_for_language(language);

    Self {
      file_path,
      language,
      queries,
    }
  }

  pub async fn symbol_stream(self) -> Result<impl Stream<Item = Symbol>> {
    let language = self.language;
    let mut parser = TreeSitterParser::with_language(self.language)?;
    let content_bytes = self.file_path.read_bytes().await?;
    let tree = parser.parse(&content_bytes, None).context("failed to create parser")?;

    let mut symbols: Vec<(usize, Symbol)> = Vec::new();

    if let Some(language_query) = self.queries {
      let language_query: &LanguageQuery = language_query;
      let symbol_index = language_query.symbol_index();

      let mut cursor = QueryCursor::new();
      let mut matches = cursor.matches(language_query.tree_sitter_query(), tree.root_node(), content_bytes.as_slice());

      while let Some(m) = StreamingIterator::next(&mut matches) {
        let meta = language_query.pattern(m.pattern_index);

        let Some(capture) = m.captures.iter().find(|c| c.index == symbol_index) else { continue };

        let node = capture.node;
        let Point { row, column } = node.start_position();

        let symbol_content_bytes = &content_bytes[node.start_byte()..node.end_byte()];
        let Some(symbol_content_str) = symbol_content_bytes.to_str() else { continue };

        let leading = meta.leading().map(|t| t.render(m, content_bytes.as_slice())).and_then(Result::ok);
        let trailing = meta.trailing().map(|t| t.render(m, content_bytes.as_slice())).and_then(Result::ok);

        #[allow(clippy::cast_possible_wrap)]
        symbols.push((
          meta.source_ordinal(),
          Symbol {
            kind: meta.kind(),
            language,
            line: row as i64 + 1,
            column: column as i64 + 1,
            content: symbol_content_str.to_string(),
            leading,
            trailing,
          },
        ));
      }
    }

    // (stable) sort by order of appearance in config
    symbols.sort_by_key(|(source_i, _)| *source_i);

    symbols.into_iter().map(|(_, symbol)| symbol).stream().ok()
  }
}
