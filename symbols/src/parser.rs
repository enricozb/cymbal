use std::{
  collections::HashSet,
  path::{Path, PathBuf},
};

use anyhow::Context;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser as TreeSitterParser, QueryCursor};

use crate::{
  config::{Config, Language, LanguageConfig},
  symbol::Symbol,
  text::{Loc, Span},
};

pub struct Parser<'a> {
  path: PathBuf,
  language: Language,
  language_config: &'a LanguageConfig,
}

impl<'a> Parser<'a> {
  pub fn from_path<P: AsRef<Path>>(config: &'a Config, path: P) -> Option<Self> {
    let path = path.as_ref();
    let extension = path.extension()?.to_str()?;
    let language = Language::from_extension(extension)?;
    let language_config = config.languages.get(&language)?;

    Some(Self {
      path: path.to_path_buf(),
      language,
      language_config,
    })
  }

  pub fn on_symbol(
    &self,
    callback: impl Fn(Symbol<(), &str>) -> Result<(), anyhow::Error>,
  ) -> Result<(), anyhow::Error> {
    let mut parser = TreeSitterParser::new();
    parser
      .set_language(&self.language.to_tree_sitter())
      .context("set_language")?;

    let content = std::fs::read_to_string(&self.path).context("read")?;

    let tree = parser.parse(content.as_bytes(), None).context("parse")?;
    let mut positions = HashSet::new();

    for (kind, queries) in &self.language_config.queries {
      for query in queries {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(query, tree.root_node(), content.as_bytes());

        while let Some(m) = matches.next() {
          for capture in m.captures {
            let node = capture.node;
            let start_pos = node.start_position();

            if positions.contains(&start_pos) {
              continue;
            } else {
              positions.insert(start_pos);
            }

            let end_pos = node.start_position();

            let start_byte = node.start_byte();
            let end_byte = node.end_byte();
            let text = &content[start_byte..end_byte];

            let span = Span::new(
              Loc::new(start_pos.row + 1, start_pos.column + 1),
              Loc::new(end_pos.row + 1, end_pos.column + 1),
            );

            callback(Symbol::new((), span, text, *kind)).context("callback")?;
          }
        }
      }
    }

    Ok(())
  }
}
