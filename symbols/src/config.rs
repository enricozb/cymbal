use std::collections::HashMap;

use serde::{de::Error, Deserialize, Deserializer};
use tree_sitter::{Language as TreeSitterLanguage, Query};

use crate::{ext::OptionExt, symbol::Kind as SymbolKind, utils::OneOrMany};

static DEFAULT_CONFIG: &str = include_str!("../default-config.toml");

#[derive(Deserialize)]
pub struct Config {
  #[serde(flatten, deserialize_with = "deserialize_languages")]
  pub languages: HashMap<Language, LanguageConfig>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Language {
  C,
  Cpp,
  Go,
  Haskell,
  Odin,
  Python,
  Rust,
  TypeScript,
}

/// A language configuration stanza.
///
/// This structure does not exactly reflect the TOML configuration. It has
/// this shape for efficiency during file parsing.
pub struct LanguageConfig {
  /// Queries for symbols that should be searched for.
  pub queries: HashMap<SymbolKind, Vec<Query>>,
  /// Transformations over query captures to format symbols.
  pub transforms: HashMap<SymbolKind, ()>,
}

impl Config {
  /// Returns all extensions the config references.
  pub fn extensions(&self) -> impl Iterator<Item = &'static str> + '_ {
    self.languages.keys().flat_map(Language::extensions).copied()
  }
}

impl Default for Config {
  fn default() -> Self {
    toml::from_str(DEFAULT_CONFIG).unwrap()
  }
}

impl Language {
  pub fn extensions(&self) -> &'static [&'static str] {
    match self {
      Self::C => &["c", "h"],
      Self::Cpp => &["cpp", "cc", "hh"],
      Self::Go => &["go"],
      Self::Haskell => &["hs"],
      Self::Odin => &["odin"],
      Self::Python => &["py"],
      Self::Rust => &["rs"],
      Self::TypeScript => &["js", "jsx", "ts", "tsx"],
    }
  }

  pub fn to_tree_sitter(&self) -> TreeSitterLanguage {
    match self {
      Self::C => tree_sitter_c::LANGUAGE.into(),
      Self::Cpp => tree_sitter_cpp::LANGUAGE.into(),
      Self::Go => tree_sitter_go::LANGUAGE.into(),
      Self::Haskell => tree_sitter_haskell::LANGUAGE.into(),
      Self::Odin => tree_sitter_odin::LANGUAGE.into(),
      Self::Python => tree_sitter_python::LANGUAGE.into(),
      Self::Rust => tree_sitter_rust::LANGUAGE.into(),
      Self::TypeScript => tree_sitter_typescript::LANGUAGE_TSX.into(),
    }
  }

  pub fn from_extension<S: AsRef<str>>(extension: S) -> Option<Self> {
    match extension.as_ref() {
      "c" | "h" => Self::C,
      "cpp" | "cc" | "hh" => Self::Cpp,
      "go" => Self::Go,
      "odin" => Self::Odin,
      "hs" => Self::Haskell,
      "py" => Self::Python,
      "rs" => Self::Rust,
      "js" | "jsx" | "ts" | "tsx" => Self::TypeScript,
      _ => return None,
    }
    .some()
  }
}

fn deserialize_languages<'de, D>(deserializer: D) -> Result<HashMap<Language, LanguageConfig>, D::Error>
where
  D: Deserializer<'de>,
{
  #[derive(Deserialize)]
  struct RawLanguageConfig {
    queries: HashMap<SymbolKind, OneOrMany<String>>,
  }

  // Deserializes queries as strings and attempts to parse them using the
  // appropriate tree-sitter language.
  HashMap::<Language, RawLanguageConfig>::deserialize(deserializer)?
    .into_iter()
    .map(|(language, language_config)| {
      let ts_language = language.to_tree_sitter();

      let queries: HashMap<SymbolKind, Vec<Query>> = language_config
        .queries
        .into_iter()
        .map(|(symbol_kind, queries)| {
          let queries = Vec::from(queries);
          let queries = queries
            .into_iter()
            .map(|query| {
              Query::new(&ts_language, &query)
                .map_err(|e| D::Error::custom(format!("failed to parse query {query:?}: {e:?}")))
            })
            .collect::<Result<_, _>>()?;

          Ok((symbol_kind, queries))
        })
        .collect::<Result<_, _>>()?;

      Ok((
        language,
        LanguageConfig {
          queries,
          transforms: Default::default(),
        },
      ))
    })
    .collect()
}
