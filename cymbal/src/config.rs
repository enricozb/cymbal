mod raw;

use std::{collections::HashMap, path::Path};

use anyhow::Result;
use clap::ValueEnum;
use indexmap::IndexMap;
use serde::Deserialize;
use sqlx::Type as SqlxType;
use tree_sitter::{Language as TreeSitterLanguage, Query as TreeSitterQuery};

use crate::ext::PathExt;
use crate::{color, config::raw::RawConfig, ext::TomlExt, symbol::Kind, template::Template, utils::Lazy};

static DEFAULT_CONFIG: &str = include_str!("../default-config.toml");

pub struct Config {
  languages: HashMap<Language, Lazy<Queries>>,
}

impl Config {
  pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
    let content = path.as_ref().read_bytes().await?;

    RawConfig::from_bytes(&content).map(Config::from)
  }

  pub fn queries_for_language(&self, language: Language) -> Option<&Lazy<Queries>> {
    self.languages.get(&language)
  }
}

impl Default for Config {
  fn default() -> Self {
    RawConfig::from_toml_str(DEFAULT_CONFIG).unwrap().into()
  }
}

pub type Queries = IndexMap<Kind, Vec<Query>>;

pub struct Query {
  ts: TreeSitterQuery,
  leading: Option<Template>,
  trailing: Option<Template>,
}

impl Query {
  pub fn tree_sitter_query(&self) -> &TreeSitterQuery {
    &self.ts
  }

  pub fn leading(&self) -> Option<&Template> {
    self.leading.as_ref()
  }

  pub fn trailing(&self) -> Option<&Template> {
    self.trailing.as_ref()
  }
}

macro_rules! Language {
  (
    $( { $display_name:literal, $name:ident, $color:ident, [$($ext:literal),*], $ts:expr } ),* $(,)?
  ) => {
    #[repr(u8)]
    #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, SqlxType, ValueEnum)]
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

      pub fn from_file_path<P: AsRef<Path>>(file_path: P) -> Option<Self> {
        Self::from_extension(file_path.as_ref().extension()?.to_str()?)
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
  { "(json)", Json, green, ["json"], tree_sitter_json::LANGUAGE.into() },
  { "(odin)", Odin, blue, ["odin"], tree_sitter_odin::LANGUAGE.into() },
  { "(ml)", Ocaml, yellow, ["ml"], tree_sitter_ocaml::LANGUAGE_OCAML.into() },
  { "(py)", Python, bright_yellow, ["py"], tree_sitter_python::LANGUAGE.into() },
  { "(rs)", Rust, yellow, ["rs"], tree_sitter_rust::LANGUAGE.into() },
  { "(ts)", TypeScript, blue, ["js", "jsx", "ts", "tsx"], tree_sitter_typescript::LANGUAGE_TSX.into() },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn config_default_no_panic() {
    Config::default();
  }
}
