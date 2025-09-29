use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use futures::Stream;
use serde::de::DeserializeOwned;
use tree_sitter::Parser as TreeSitterParser;

use crate::{config::Language, utils::Lazy};

pub trait Is<T> {
  fn is(self) -> T;
}

impl<T> Is<T> for T {
  fn is(self) -> T {
    self
  }
}

#[extend::ext(name=IntoExt)]
pub impl<T> T {
  fn some(self) -> Option<T> {
    Some(self)
  }

  fn ok<E>(self) -> Result<T, E> {
    Ok(self)
  }

  fn convert<U: From<T>>(self) -> U {
    self.into()
  }

  fn ready(self) -> impl Future<Output = T> {
    std::future::ready(self)
  }
}

#[extend::ext(name=ResultExt)]
pub impl<T, E> Result<T, E> {
  fn into_anyhow(self) -> Result<T>
  where
    E: std::error::Error + Sync + Send + 'static,
  {
    self?.ok()
  }
}

#[extend::ext(name=IteratorExt)]
pub impl<T: IntoIterator> T {
  fn ok_all<U, E>(self) -> Result<Vec<U>, E>
  where
    T::Item: Into<Result<U, E>>,
  {
    self.into_iter().map(Into::into).collect()
  }

  fn stream(self) -> impl Stream<Item = <T::IntoIter as std::iter::Iterator>::Item> {
    futures::stream::iter(self)
  }
}

#[extend::ext(name=TryStreamExt)]
pub impl<T, E, S: Stream<Item = Result<T, E>>> S {
  fn filter_ok(self) -> impl Stream<Item = T> {
    use futures::StreamExt;

    self.filter_map(|res| async move { res.ok() })
  }
}

#[extend::ext(name=OptionExt)]
pub impl<T> Option<T> {
  async fn into_future<U>(self) -> Option<U>
  where
    T: Future<Output = U>,
  {
    match self {
      Some(x) => Some(x.await),
      None => None,
    }
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

#[extend::ext(name=PathBufExt)]
pub impl PathBuf {
  fn into_bytes(self) -> Vec<u8> {
    self.into_os_string().into_encoded_bytes()
  }
}

#[extend::ext(name=PathExt)]
pub impl<T> T {
  fn as_bytes<'a>(self) -> &'a [u8]
  where
    Self: Is<&'a Path>,
  {
    self.is().as_os_str().as_encoded_bytes()
  }

  async fn read_bytes(&self) -> Result<Vec<u8>>
  where
    Self: AsRef<Path>,
  {
    tokio::fs::read(self).await.context("failed to read")
  }
}

#[extend::ext(name=StrExt)]
pub impl<'a> &'a [u8] {
  fn to_str(&self) -> Option<&'a str> {
    std::str::from_utf8(self).ok()
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

#[extend::ext(name=LazyExt)]
pub impl<T> Lazy<T> {
  fn take(self) -> T {
    match Self::into_inner(self) {
      Ok(value) => value,
      Err(f) => f(),
    }
  }
}
