mod raw;

use std::{collections::HashMap, path::Path};

use anyhow::Result;
use clap::ValueEnum;
use enum_assoc::Assoc;
use indexmap::IndexMap;
use serde::Deserialize;
use sqlx::Type as SqlxType;
use tree_sitter::Query as TreeSitterQuery;

use crate::{
  color,
  config::raw::RawConfig,
  ext::{PathExt, TomlExt},
  symbol::Kind,
  template::Template,
  utils::Lazy,
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

#[derive(Assoc, Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, SqlxType, ValueEnum)]
#[serde(rename_all = "lowercase")]
#[func(pub fn from_extension(s: &str) -> Option<Self>)]
#[func(pub const fn colored(&self) -> &'static str)]
pub enum Language {
  #[assoc(colored = color!("c   ", blue), from_extension = "c" | "h")]
  C,
  #[assoc(colored = color!("c++ ", blue), from_extension = "cpp" | "cc" | "hh")]
  CPP,
  #[assoc(colored = color!("fish", green), from_extension = "fish")]
  Fish,
  #[assoc(colored = color!("go  ", cyan), from_extension = "go")]
  Go,
  #[assoc(colored = color!("hs  ", magenta), from_extension = "hs")]
  Haskell,
  #[assoc(colored = color!("json", green), from_extension = "json")]
  JSON,
  #[assoc(colored = color!("flow", yellow), from_extension = "ml")]
  OCaml,
  #[assoc(colored = color!("odin", blue), from_extension = "odin")]
  Odin,
  #[assoc(colored = color!("py  ", bright_yellow), from_extension = "py")]
  Python,
  #[assoc(colored = color!("rs  ", yellow), from_extension = "rs")]
  Rust,
  #[assoc(colored = color!("ts  ", blue), from_extension = "js" | "jsx")]
  JavaScript,
  #[assoc(colored = color!("ts  ", blue), from_extension = "ts")]
  TypeScript,
  #[assoc(colored = color!("tsx ", blue), from_extension = "tsx")]
  TSX,
  #[assoc(colored = color!("ivy ", green), from_extension = "iv")]
  Ivy,
  #[assoc(colored = color!("vine", green), from_extension = "vi")]
  Vine,
  #[assoc(colored = color!("kak ", green), from_extension = "kak")]
  Kak,
}

languages_impl!(Language);

impl Language {
  pub fn from_file_path<P: AsRef<Path>>(file_path: P) -> Option<Self> {
    Self::from_extension(file_path.as_ref().extension()?.to_str()?)
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
