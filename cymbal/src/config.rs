mod raw;

use std::{collections::HashMap, ffi::OsStr, path::Path};

use anyhow::Result;
use clap::ValueEnum;
use enum_assoc::Assoc;
use indexmap::IndexMap;
use serde::Deserialize;
use sqlx::Type as SqlxType;
use tree_sitter::Query as TreeSitterQuery;

use crate::{
  color::{BLUE, BRIGHT_YELLOW, CYAN, GREEN, MAGENTA, YELLOW},
  config::raw::RawConfig,
  ext::{PathExt, TomlExt},
  symbol::Kind,
  template::Template,
  utils::{Colored, Lazy},
};

include!(concat!(env!("OUT_DIR"), "/", "grammars.rs"));

static DEFAULT_CONFIG: &str = include_str!("../default-config.toml");

pub struct Config {
  languages: HashMap<Language, Lazy<Queries>>,
}

impl Config {
  pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
    let content = path.as_ref().read_bytes().await?;

    RawConfig::from_bytes(&content).map(Config::from)
  }

  pub fn contains_language(&self, language: Language) -> bool {
    self.languages.contains_key(&language)
  }

  pub fn for_language(self, language: Language) -> Self {
    Self {
      languages: self
        .languages
        .into_iter()
        .filter(|(config_lang, _)| config_lang == &language)
        .collect(),
    }
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

#[allow(clippy::unsafe_derive_deserialize)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Assoc, Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, SqlxType, ValueEnum)]
#[serde(rename_all = "lowercase")]
#[value(rename_all = "lowercase")]
#[func(pub fn from_extension(s: &str) -> Option<Self>)]
#[func(pub fn from_file_name(s: &str) -> Option<Self> { None })]
#[func(pub const fn to_str(&self) -> &'static str)]
#[func(pub const fn color(&self) -> &'static str)]
pub enum Language {
  #[assoc(to_str = "c   ", color = BLUE, from_extension = "c" | "h")]
  C,
  #[assoc(to_str = "c++ ", color = BLUE, from_extension = "cpp" | "cc" | "hh")]
  CPP,
  #[assoc(to_str = "fish", color = GREEN, from_extension = "fish")]
  Fish,
  #[assoc(to_str = "go  ", color = CYAN, from_extension = "go")]
  Go,
  #[assoc(to_str = "hs  ", color = MAGENTA, from_extension = "hs")]
  Haskell,
  #[assoc(to_str = "json", color = GREEN, from_extension = "json")]
  JSON,
  #[assoc(to_str = "caml", color = YELLOW, from_extension = "ml")]
  OCaml,
  #[assoc(to_str = "odin", color = BLUE, from_extension = "odin")]
  Odin,
  #[assoc(to_str = "py  ", color = BRIGHT_YELLOW, from_extension = "py")]
  Python,
  #[assoc(to_str = "rs  ", color = YELLOW, from_extension = "rs")]
  Rust,
  #[assoc(to_str = "js  ", color = BLUE, from_extension = "js" | "jsx")]
  JavaScript,
  #[assoc(to_str = "ts  ", color = BLUE, from_extension = "ts" | "tsx")]
  #[serde(alias = "typescript")]
  TSX,
  #[assoc(to_str = "ivy ", color = GREEN, from_extension = "iv")]
  Ivy,
  #[assoc(to_str = "vine", color = GREEN, from_extension = "vi")]
  Vine,
  #[assoc(to_str = "kak ", color = GREEN, from_extension = "kak", from_file_name = "kakrc")]
  Kak,
  #[assoc(to_str = "nu  ", color = BLUE, from_extension = "nu")]
  Nu,
}

languages_impl!(Language);

impl Language {
  pub fn from_file_path<P: AsRef<Path>>(file_path: P) -> Option<Self> {
    let file_path = file_path.as_ref();
    let file_name = file_path.file_name().and_then(OsStr::to_str);
    let extension = file_path.extension().and_then(OsStr::to_str);

    extension
      .and_then(Self::from_extension)
      .or_else(|| file_name.and_then(Self::from_file_name))
  }

  pub const fn colored(&self, color: bool) -> Colored {
    Colored {
      string: self.to_str(),
      color: if color { Some(self.color()) } else { None },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn config_default_no_panic() {
    Config::default();
  }
}
