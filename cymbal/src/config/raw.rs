use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::Deserialize;
use tree_sitter::{Language as TreeSitterLanguage, Query as TreeSitterQuery};

use crate::{
  config::{Config, Language, Queries, Query, Template},
  ext::{HashMapExt, IntoExt, LazyExt},
  symbol::Kind,
  utils::{Lazy, OneOrMany},
};

/// A config that explicitly represents the shape of the config TOML file.
/// This is intended to be filtered and parsed into a [`Config`].
#[derive(Deserialize)]
pub struct RawConfig {
  pub inherit: Option<Inherit>,
  #[serde(flatten)]
  pub languages: HashMap<Language, RawLanguageQueries>,
}

impl RawConfig {
  fn inherited_config(&self) -> HashMap<Language, Lazy<Queries>> {
    let Some(inherit) = &self.inherit else { return HashMap::default() };

    match inherit {
      Inherit::All(false) => HashMap::default(),
      Inherit::All(true) => Config::default().languages,
      Inherit::Languages(languages) => Config::default().languages.restrict(languages),
    }
  }

  fn provided_config(self) -> HashMap<Language, Lazy<Queries>> {
    self
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
      .collect()
  }

  fn merge_inherited_and_provided_queries(inherited: Lazy<Queries>, provided: Lazy<Queries>) -> Lazy<Queries> {
    Lazy::new(Box::new(move || {
      let inherited = Lazy::take(inherited);
      let mut provided = Lazy::take(provided);

      for (kind, inherited_queries) in inherited {
        let Some(provided_queries) = provided.get_mut(&kind) else {
          provided.insert(kind, inherited_queries);
          continue;
        };

        provided_queries.extend(inherited_queries);
      }

      provided
    }))
  }

  fn merge_inherited_and_provided_configs(
    inherited: HashMap<Language, Lazy<Queries>>,
    mut provided: HashMap<Language, Lazy<Queries>>,
  ) -> HashMap<Language, Lazy<Queries>> {
    for (language, inherited_queries) in inherited {
      let Some(provided_queries) = provided.remove(&language) else {
        provided.insert(language, inherited_queries);
        continue;
      };

      provided.insert(
        language,
        Self::merge_inherited_and_provided_queries(inherited_queries, provided_queries),
      );
    }

    provided
  }
}

impl From<RawConfig> for Config {
  fn from(raw_config: RawConfig) -> Self {
    let inherited_config = raw_config.inherited_config();
    let provided_config = raw_config.provided_config();
    let languages = RawConfig::merge_inherited_and_provided_configs(inherited_config, provided_config);

    Self { languages }
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Inherit {
  All(bool),
  Languages(HashSet<Language>),
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
