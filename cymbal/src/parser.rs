use std::{
  collections::HashSet,
  path::{Path, PathBuf},
};

use anyhow::Context;
use indexmap::IndexMap;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser as TreeSitterParser, QueryCursor, QueryMatch};

use crate::{
  config::{Config, Language, Query},
  symbol::{Kind as SymbolKind, Symbol},
  text::{Loc, Span},
};

pub struct Parser<'a> {
  path: PathBuf,
  queries: &'a IndexMap<SymbolKind, Vec<Query>>,
  pub language: Language,
}

impl<'a> Parser<'a> {
  pub fn from_path<P: AsRef<Path>>(config: &'a Config, path: P) -> Option<Self> {
    let path = path.as_ref();
    let extension = path.extension()?.to_str()?;
    let language = Language::from_extension(extension)?;
    let queries = &config.languages.get(&language)?.queries;

    Some(Self {
      path: path.to_path_buf(),
      language,
      queries,
    })
  }

  pub fn on_symbol(
    &self,
    callback: impl Fn(Symbol<(), &str>) -> Result<(), anyhow::Error>,
  ) -> Result<(), anyhow::Error> {
    let mut parser = TreeSitterParser::new();
    parser
      .set_language(&self.language.as_tree_sitter())
      .context("set_language")?;

    let content = std::fs::read_to_string(&self.path).context("read")?;

    let tree = parser.parse(content.as_bytes(), None).context("parse")?;
    let mut positions = HashSet::new();

    for (kind, queries) in self.queries {
      for query in queries {
        let Some(symbol_index) = query.ts.capture_index_for_name("symbol") else {
          continue;
        };

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query.ts, tree.root_node(), content.as_bytes());

        while let Some(m) = matches.next() {
          let Some(capture) = m.captures.iter().find(|q| q.index == symbol_index) else {
            continue;
          };

          let node = capture.node;
          let start_pos = node.start_position();

          if positions.contains(&start_pos) {
            continue;
          }
          positions.insert(start_pos);

          let end_pos = node.start_position();

          let start_byte = node.start_byte();
          let end_byte = node.end_byte();
          let text = &content[start_byte..end_byte];

          let span = Span::new(
            Loc::new(start_pos.row + 1, start_pos.column + 1),
            Loc::new(end_pos.row + 1, end_pos.column + 1),
          );

          let lead = &query.render_leading(m, &content).context("failed to render leading")?;
          let tail = &query
            .render_trailing(m, &content)
            .context("failed to render trailing")?;

          callback(Symbol {
            path: (),
            span,
            lead,
            text,
            tail,
            kind: *kind,
          })
          .context("callback")?;
        }
      }
    }

    Ok(())
  }
}

impl Query {
  pub fn render_leading(&self, m: &QueryMatch, content: &str) -> Result<String, anyhow::Error> {
    let Some(leading) = &self.leading else {
      return Ok(String::new());
    };

    leading.render(m, content).context("failed to render")
  }

  pub fn render_trailing(&self, m: &QueryMatch, content: &str) -> Result<String, anyhow::Error> {
    let Some(trailing) = &self.trailing else {
      return Ok(String::new());
    };

    trailing.render(m, content).context("failed to render")
  }
}
