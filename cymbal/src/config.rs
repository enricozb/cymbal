pub mod raw;

use std::collections::HashMap;

use clap::ValueEnum;
use serde::Deserialize;
use tree_sitter::{Language as TreeSitterLanguage, Query as TreeSitterQuery};

use crate::{ext::OptionExt, symbol::Kind as SymbolKind, template::Template};

static DEFAULT_CONFIG: &str = include_str!("../default-config.toml");

pub struct Config {
  pub languages: HashMap<Language, LanguageQueries>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, ValueEnum)]
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

pub struct LanguageQueries {
  /// Queries for symbols that should be searched for.
  pub queries: HashMap<SymbolKind, Vec<Query>>,
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

impl Language {
  pub fn extensions(self) -> &'static [&'static str] {
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

  pub fn as_tree_sitter(self) -> TreeSitterLanguage {
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
