pub mod raw;

use std::{collections::HashMap, sync::LazyLock};

use clap::ValueEnum;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tree_sitter::{Language as TreeSitterLanguage, Query as TreeSitterQuery};

use crate::{color, symbol::Kind as SymbolKind, template::Template};

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
    $( { $display_name:literal, $name:ident, $color:ident, [$($ext:literal),*], $ts:expr } ),* $(,)?
  ) => {
    #[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, ValueEnum)]
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

      pub fn colored_abbreviation(self) -> &'static str {
        match self {
          $( Self::$name => color!($display_name, $color), )*
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
  { "(c)", C, blue, ["c", "h"], tree_sitter_c::LANGUAGE.into() },
  { "(c++)", Cpp, blue, ["cpp", "cc", "hh"], tree_sitter_cpp::LANGUAGE.into() },
  { "(fish)", Fish, green, ["fish"], tree_sitter_fish::language() },
  { "(go)", Go, cyan, ["go"], tree_sitter_go::LANGUAGE.into() },
  { "(hs)", Haskell, magenta, ["hs"], tree_sitter_haskell::LANGUAGE.into() },
  { "(odin)", Odin, blue, ["odin"], tree_sitter_odin::LANGUAGE.into() },
  { "(ml)", Ocaml, yellow, ["ml"], tree_sitter_ocaml::LANGUAGE_OCAML.into() },
  { "(py)", Python, bright_yellow, ["py"], tree_sitter_python::LANGUAGE.into() },
  { "(rs)", Rust, yellow, ["rs"], tree_sitter_rust::LANGUAGE.into() },
  { "(ts)", TypeScript, blue, ["js", "jsx", "ts", "tsx"], tree_sitter_typescript::LANGUAGE_TSX.into() },
}
