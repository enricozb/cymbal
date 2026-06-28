use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use serde::Deserialize;

use crate::{
  config::{Config, Language, LanguageQuery, Queries, QuerySource},
  ext::{HashMapExt, TomlExt},
  symbol::Kind,
  utils::{Lazy, OneOrMany},
};

pub static DEFAULT_CONFIG: &str = include_str!("../../default-config.toml");

fn default_queries() -> HashMap<Language, Queries> {
  RawConfig::from_toml_str(DEFAULT_CONFIG)
    .expect("failed to parse default config")
    .provided_config()
}

/// A config that explicitly represents the shape of the config TOML file.
/// This is intended to be filtered and parsed into a [`Config`].
#[derive(Deserialize)]
pub struct RawConfig {
  pub inherit: Option<Inherit>,
  #[serde(flatten)]
  pub languages: HashMap<Language, RawLanguageQueries>,
}

impl RawConfig {
  /// The parts of the default config being inherited.
  fn inherited_config(&self) -> HashMap<Language, Queries> {
    let Some(inherit) = &self.inherit else { return HashMap::default() };

    match inherit {
      Inherit::All(false) => HashMap::default(),
      Inherit::All(true) => default_queries(),
      Inherit::Languages(languages) => default_queries().restrict(languages),
    }
  }

  /// The parts of the config explicitly provided.
  fn provided_config(self) -> HashMap<Language, Queries> {
    self
      .languages
      .into_iter()
      .map(|(language, language_config)| {
        let queries = language_config
          .queries
          .into_iter()
          .map(|(symbol_kind, queries)| {
            let queries = queries.into_iter().map(RawQuery::into_query).collect();

            (symbol_kind, queries)
          })
          .collect();

        (language, queries)
      })
      .collect()
  }

  fn merge_inherited_and_provided_queries(inherited: Queries, mut provided: Queries) -> Queries {
    for (kind, inherited_queries) in inherited {
      let Some(provided_queries) = provided.get_mut(&kind) else {
        provided.insert(kind, inherited_queries);
        continue;
      };

      provided_queries.extend(inherited_queries);
    }

    provided
  }

  fn merge_inherited_and_provided_configs(
    inherited: HashMap<Language, Queries>,
    mut provided: HashMap<Language, Queries>,
  ) -> HashMap<Language, Queries> {
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
    let merged = RawConfig::merge_inherited_and_provided_configs(inherited_config, provided_config);

    let languages = merged
      .into_iter()
      .map(|(language, queries)| {
        let combined = Lazy::new(Box::new(move || {
          LanguageQuery::build(language, queries).expect("failed to build combined query")
        }));

        (language, combined)
      })
      .collect();

    Config { languages }
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
  fn into_query(self) -> QuerySource {
    match self {
      Self::Bare(query) => QuerySource {
        source: query,
        leading: None,
        trailing: None,
      },

      Self::WithContext { leading, query, trailing } => QuerySource {
        source: query,
        leading,
        trailing,
      },
    }
  }
}
