pub mod raw;

use std::{collections::HashMap, sync::LazyLock};

use clap::ValueEnum;
use indexmap::IndexMap;
use serde::Deserialize;
use tree_sitter::{Language as TreeSitterLanguage, Query as TreeSitterQuery};

use crate::{symbol::Kind as SymbolKind, template::Template};

static DEFAULT_CONFIG: &str = include_str!("../default-config.toml");

type Lazy<T> = LazyLock<T, Box<dyn FnOnce() -> T + Send>>;

pub struct Config {
  pub languages: HashMap<Language, Lazy<LanguageQueries>>,
}

pub struct LanguageQueries {
  /// Queries for symbols that should be searched for.
  pub queries: IndexMap<SymbolKind, Vec<Query>>,
}

pub struct Query {
  pub ts: TreeSitterQuery,
  pub leading: Option<Template>,
  pub trailing: Option<Template>,
}

impl Config {
  /// Returns all extensions the config references.
  pub fn extensions(&self) -> impl Iterator<Item = &'static str> + '_ {
    self.languages.keys().flat_map(|l| l.extensions()).copied()
  }
}

macro_rules! Language {
  (
    $( { $name:ident, [$($ext:literal),*], $ts:expr } ),* $(,)?
  ) => {
    #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, ValueEnum)]
    #[serde(rename_all = "lowercase")]
    pub enum Language {
      $( $name, )*
    }

    impl Language {
      pub fn extensions(self) -> &'static [&'static str] {
        match self {
          $( Self::$name => &[$($ext),*], )*
        }
      }

      pub fn as_tree_sitter(self) -> TreeSitterLanguage {
        match self {
          $( Self::$name => $ts, )*
        }
      }

      pub fn from_extension<S: AsRef<str>>(extension: S) -> Option<Self> {
        match extension.as_ref() {
          $(
            $( $ext => Some(Self::$name), )*
          )*
          _ => None,
        }
      }
    }
  };
}

Language! {
  { C, ["c", "h"], tree_sitter_c::LANGUAGE.into() },
  { Cpp, ["cpp", "cc", "hh"], tree_sitter_cpp::LANGUAGE.into() },
  { Fish, ["fish"], tree_sitter_fish::language() },
  { Go, ["go"], tree_sitter_go::LANGUAGE.into() },
  { Haskell, ["hs"], tree_sitter_haskell::LANGUAGE.into() },
  { Odin, ["odin"], tree_sitter_odin::LANGUAGE.into() },
  { Python, ["py"], tree_sitter_python::LANGUAGE.into() },
  { Rust, ["rs"], tree_sitter_rust::LANGUAGE.into() },
  { TypeScript, ["js", "jsx", "ts", "tsx"], tree_sitter_typescript::LANGUAGE_TSX.into() },
}
