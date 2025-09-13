use std::path::Path;

use anyhow::{Context, Result};
use futures::Stream;
use serde::de::DeserializeOwned;
use tree_sitter::Parser as TreeSitterParser;

use crate::config::Language;

#[extend::ext(name=ResultExt)]
pub impl<T> T {
  fn ok<E>(self) -> Result<T, E> {
    Ok(self)
  }

  fn into_anyhow<O, E>(self) -> Result<O>
  where
    E: std::error::Error + Sync + Send + 'static,
    Self: Into<Result<O, E>>,
  {
    self.into()?.ok()
  }
}

#[extend::ext(name=OptionExt)]
pub impl<T> T {
  fn some(self) -> Option<T> {
    Some(self)
  }
}

#[extend::ext(name=IntoStream)]
pub impl<T: IntoIterator> T {
  fn stream(self) -> impl Stream<Item = <T::IntoIter as std::iter::Iterator>::Item> {
    futures::stream::iter(self)
  }
}

#[extend::ext(name=Leak)]
pub impl<T> T {
  fn leak(self) -> &'static T {
    Box::leak(Box::new(self))
  }
}

#[extend::ext(name=TomlExt)]
pub impl<T: DeserializeOwned> T {
  fn from_toml_str(toml_str: &str) -> Result<T> {
    toml::from_str(toml_str).context("failed to parse str")
  }

  fn from_bytes(toml_bytes: &[u8]) -> Result<T> {
    toml::from_slice(toml_bytes).context("failed to parse bytes")
  }
}

#[extend::ext(name=Ignore)]
pub impl<T> T {
  fn ignore(self) {}
}

#[extend::ext(name=PathExt)]
pub impl<T: AsRef<Path>> T {
  fn into_owned_string_lossy(&self) -> String {
    self.as_ref().to_string_lossy().into_owned()
  }

  async fn read_bytes(&self) -> Result<Vec<u8>> {
    tokio::fs::read(self).await.context("failed to read")
  }
}

#[extend::ext(name=IntoExt)]
pub impl<T> T {
  fn convert<U: From<T>>(self) -> U {
    self.into()
  }
}

#[extend::ext(name=StrExt)]
pub impl<'a> &'a [u8] {
  fn to_str(&self) -> Option<&'a str> {
    std::str::from_utf8(&self).ok()
  }
}

#[extend::ext(name=TreeSitterParserExt)]
pub impl TreeSitterParser {
  fn with_language(language: Language) -> Result<TreeSitterParser> {
    let mut parser = TreeSitterParser::new();

    parser
      .set_language(&language.as_tree_sitter())
      .context("failed to set tree sitter parser language")?;

    parser.ok()
  }
}
