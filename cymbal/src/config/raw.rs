use std::collections::HashMap;

use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::Deserialize;
use tree_sitter::{Language as TreeSitterLanguage, Query as TreeSitterQuery};

use crate::config::{Config, Language, Query, Template};
use crate::ext::ResultExt;
use crate::symbol::Kind;
use crate::utils::{Lazy, OneOrMany};

/// A config that explicitly represents the shape of the config TOML file.
/// This is intended to be filtered and parsed into a [`Config`].
#[derive(Deserialize)]
pub struct RawConfig {
  #[serde(flatten)]
  pub languages: HashMap<Language, RawLanguageQueries>,
}

impl From<RawConfig> for Config {
  fn from(raw_config: RawConfig) -> Self {
    let languages = raw_config
      .languages
      .into_iter()
      .map(|(language, language_config)| {
        (
          language,
          Lazy::new(Box::new(move || {
            let ts_language = language.as_tree_sitter();

            language_config
              .queries
              .into_iter()
              .map(|(symbol_kind, queries)| {
                let queries = queries
                  .into_iter()
                  .map(|raw_query| raw_query.into_query(&ts_language))
                  .collect::<Result<_>>()?;

                (symbol_kind, queries).ok()
              })
              .collect::<Result<_>>()
              .unwrap()
          })),
        )
      })
      .collect();

    Self { languages }
  }
}

#[derive(Clone, Deserialize)]
pub struct RawLanguageQueries {
  #[serde(flatten)]
  queries: IndexMap<Kind, OneOrMany<RawQuery>>,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum RawQuery {
  /// Queries which have no leading/trailing text, and result in symbols defined
  /// exactly by text in the captured by the `@symbol` named capture.
  Bare(String),
  /// Queries which have custom named captures other than `@symbol` (such as
  /// `@class`, `@trait`), which are then referenced in the leading/trailing
  /// template strings. See the [`Config`] and [`Template`] docs for details.
  WithContext {
    query: String,
    #[serde(default)]
    leading: Option<String>,
    #[serde(default)]
    trailing: Option<String>,
  },
}

impl RawQuery {
  fn into_query(self, ts_language: &TreeSitterLanguage) -> Result<Query, anyhow::Error> {
    match self {
      Self::Bare(query) => Query {
        ts: TreeSitterQuery::new(ts_language, &query).context("failed to parse query")?,
        leading: None,
        trailing: None,
      },

      Self::WithContext { leading, query, trailing } => {
        let ts = TreeSitterQuery::new(ts_language, &query).context("failed to parse query")?;

        Query {
          leading: leading.map(|t| Template::parse(t, &ts).context("leading")).transpose()?,
          trailing: trailing.map(|t| Template::parse(t, &ts).context("trailing")).transpose()?,
          ts,
        }
      }
    }
    .ok()
  }
}
