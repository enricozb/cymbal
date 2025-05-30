use std::collections::HashMap;

use anyhow::Context;
use indexmap::IndexMap;
use serde::Deserialize;
use tree_sitter::{Language as TreeSitterLanguage, Query as TreeSitterQuery};

use crate::{
  config::{Config, Language, LanguageQueries, Lazy, Query},
  ext::{OptionExt, ResultExt},
  symbol::Kind as SymbolKind,
  template::Template,
  utils::OneOrMany,
};

/// A config that explicitly represents the shape of the config TOML file.
/// This is intended to be filtered and parsed into a [`Config`].
#[derive(Deserialize)]
pub struct RawConfig {
  #[serde(flatten)]
  pub languages: HashMap<Language, RawLanguageQueries>,
}

#[derive(Clone, Deserialize)]
pub struct RawLanguageQueries {
  #[serde(flatten)]
  queries: IndexMap<SymbolKind, OneOrMany<RawQuery>>,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum RawQuery {
  Bare(String),
  Transformed {
    query: String,
    #[serde(default)]
    leading: String,
    #[serde(default)]
    trailing: String,
  },
}

impl RawConfig {
  /// Returns a copy of this config containing only the provided language.
  pub fn for_language(&self, language: Language) -> Self {
    Self {
      languages: self
        .languages
        .iter()
        .filter(|(l, _)| *l == &language)
        .filter_map(|(l, qs)| if *l == language { (*l, qs.clone()).some() } else { None })
        .collect(),
    }
  }
}

impl Default for RawConfig {
  fn default() -> Self {
    toml::from_str(super::DEFAULT_CONFIG).unwrap()
  }
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

            let queries: IndexMap<SymbolKind, Vec<Query>> = language_config
              .queries
              .into_iter()
              .map(|(symbol_kind, queries)| {
                let queries = queries
                  .into_iter()
                  .map(|raw_query| raw_query.into_query(&ts_language))
                  .collect::<Result<_, _>>()?;

                Ok::<_, anyhow::Error>((symbol_kind, queries))
              })
              .collect::<Result<_, _>>()
              .unwrap();

            LanguageQueries { queries }
          })),
        )
      })
      .collect();

    Self { languages }
  }
}

impl RawQuery {
  fn into_query(self, ts_language: &TreeSitterLanguage) -> Result<Query, anyhow::Error> {
    match self {
      Self::Bare(query) => Query {
        ts: TreeSitterQuery::new(ts_language, &query).context("failed to parse query")?,
        leading: None,
        trailing: None,
      },

      Self::Transformed {
        leading,
        query,
        trailing,
      } => {
        let ts = TreeSitterQuery::new(ts_language, &query).context("failed to parse query")?;

        Query {
          leading: Template::parse(leading, &ts).context("leading")?.some(),
          trailing: Template::parse(trailing, &ts).context("trailing")?.some(),
          ts,
        }
      }
    }
    .ok()
  }
}
