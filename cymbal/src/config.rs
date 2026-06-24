mod raw;

use std::{collections::HashMap, ffi::OsStr, path::Path};

use anyhow::{Context, Result};
use clap::ValueEnum;
use enum_assoc::Assoc;
use indexmap::IndexMap;
use serde::Deserialize;
use sqlx::Type as SqlxType;
use tree_sitter::Query as TreeSitterQuery;

use crate::{
  color::{BLUE, BRIGHT_YELLOW, CYAN, GREEN, MAGENTA, YELLOW},
  config::raw::{DEFAULT_CONFIG, RawConfig},
  ext::{PathExt, TomlExt},
  symbol::Kind,
  template::Template,
  utils::{Colored, Lazy},
};

include!(concat!(env!("OUT_DIR"), "/", "grammars.rs"));

pub struct Config {
  languages: HashMap<Language, Lazy<LanguageQuery>>,
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

  pub fn queries_for_language(&self, language: Language) -> Option<&Lazy<LanguageQuery>> {
    self.languages.get(&language)
  }
}

impl Default for Config {
  fn default() -> Self {
    RawConfig::from_toml_str(DEFAULT_CONFIG).unwrap().into()
  }
}

/// The intermediate, mergeable representation of a language's queries, keyed by
/// [`Kind`]. Each [`QuerySource`] keeps the raw query/template text so that all
/// of a language's queries can later be combined into a single
/// [`LanguageQuery`].
pub type Queries = IndexMap<Kind, Vec<QuerySource>>;

/// The raw text of a single configured query and its optional leading/trailing
/// templates, before being combined and compiled.
#[derive(Clone)]
pub struct QuerySource {
  pub source: String,
  pub leading: Option<String>,
  pub trailing: Option<String>,
}

/// All of a language's queries compiled into a single tree-sitter [`Query`].
///
/// Combining every pattern into one query lets us extract all symbols in a
/// single tree walk instead of one walk per configured query. Per-pattern
/// metadata (kind and leading/trailing templates) is recovered at match time
/// via [`tree_sitter::QueryMatch::pattern_index`].
pub struct LanguageQuery {
  ts: TreeSitterQuery,
  symbol_index: u32,
  /// Indexed by tree-sitter pattern index.
  patterns: Vec<PatternMeta>,
}

pub struct PatternMeta {
  kind: Kind,
  /// The ordinal of the configured query this pattern originated from. Used to
  /// emit symbols in configuration order (which encodes kind/query precedence)
  /// rather than tree order.
  source_ordinal: usize,
  leading: Option<Template>,
  trailing: Option<Template>,
}

impl PatternMeta {
  pub fn kind(&self) -> Kind {
    self.kind
  }

  pub fn source_ordinal(&self) -> usize {
    self.source_ordinal
  }

  pub fn leading(&self) -> Option<&Template> {
    self.leading.as_ref()
  }

  pub fn trailing(&self) -> Option<&Template> {
    self.trailing.as_ref()
  }
}

impl LanguageQuery {
  /// Combines all of a language's [`Queries`] into a single compiled query.
  pub fn build(language: Language, queries: Queries) -> Result<Self> {
    let ts_language = language.as_tree_sitter_language();

    let mut source = String::new();
    // (kind, source_ordinal, leading, trailing) for each configured query.
    //
    // NOTE: each query entry must contain exactly one tree-sitter query
    let mut metas: Vec<(Kind, usize, Option<String>, Option<String>)> = Vec::new();

    for (source_i, (kind, query_source)) in queries
      .into_iter()
      .flat_map(|(kind, query_sources)| query_sources.into_iter().map(move |query_source| (kind, query_source)))
      .enumerate()
    {
      source.push_str(&query_source.source);
      source.push('\n');

      metas.push((kind, source_i, query_source.leading, query_source.trailing));
    }

    let ts = TreeSitterQuery::new(&ts_language, &source).context("failed to parse combined query")?;

    anyhow::ensure!(
      ts.pattern_count() == metas.len(),
      "each query entry must contain exactly one pattern (found {} patterns across {} entries)",
      ts.pattern_count(),
      metas.len(),
    );

    let symbol_index = ts
      .capture_index_for_name("symbol")
      .context("combined query has no @symbol capture")?;

    let patterns = metas
      .into_iter()
      .map(|(kind, source_ordinal, leading, trailing)| {
        // Templates resolve capture names against the *combined* query, so a
        // name like `{scope}` maps to its single global capture index.
        Ok(PatternMeta {
          kind,
          source_ordinal,
          leading: leading.map(|t| Template::parse(t, &ts).context("leading")).transpose()?,
          trailing: trailing.map(|t| Template::parse(t, &ts).context("trailing")).transpose()?,
        })
      })
      .collect::<Result<_>>()?;

    Ok(Self {
      ts,
      symbol_index,
      patterns,
    })
  }

  pub fn tree_sitter_query(&self) -> &TreeSitterQuery {
    &self.ts
  }

  pub fn symbol_index(&self) -> u32 {
    self.symbol_index
  }

  pub fn pattern(&self, index: usize) -> &PatternMeta {
    &self.patterns[index]
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
  #[assoc(to_str = "c++ ", color = BLUE, from_extension = "cpp" | "cc" | "hpp" | "hh")]
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
