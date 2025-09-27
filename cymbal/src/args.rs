use std::num::NonZero;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use crate::cache::Cache;
use crate::channel::{Receiver, Sender};
use crate::config::{Config, Language};
use crate::ext::{IntoExt, OptionExt};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  /// A toml file with language queries and symbols.
  ///
  /// The default configuration will be applied if this argument is not
  /// provided.
  #[arg(short, long = "config")]
  config_path: Option<PathBuf>,

  /// The file or directory to search for symbols in.
  ///
  /// If this is a directory, it is recursively searched for files with
  /// supported extensions.
  ///
  /// If this is a file, it is searched for symbols, and the `--language` flag
  /// is ignored, and the language appropriate for the file is used.
  #[arg(default_value = ".")]
  search_path: PathBuf,

  /// The characters between properties of a single symbol.
  ///
  /// This is the character between the file path, location, kind, text, and
  /// leading/trailing text written to stdout.
  ///
  /// This defaults to U+200B (zero-width space).
  #[arg(short, long, default_value_t = '\u{200B}')]
  delimiter: char,

  /// The character between symbols.
  ///
  /// This defaults to U+0 (null byte).
  #[arg(short, long, default_value_t = '\0')]
  separator: char,

  /// Only show symbols from files with extensions matching this language.
  ///
  /// This flag takes precedence over the `--extension` flag.
  #[arg(long)]
  language: Option<Language>,

  /// Only show symbols from files with the language matching this extension.
  ///
  /// The `--language` flag takes precedence over this flag.
  #[arg(long)]
  extension: Option<String>,

  /// Directory to cache parsed symbols.
  ///
  /// Files are reparsed if their cached mtime differs from than their current
  /// mtime, or the path of the file doesn't exist in the cache. This option
  /// is typically used when `symbols` is called from the same directory
  /// multiple times, such as searching over a code base in an editor.
  #[arg(long = "cache")]
  cache_dirpath: Option<PathBuf>,

  /// The number of parser tasks, or roughly the amount of parallelism.
  #[arg(long)]
  concurrency: Option<NonZero<usize>>,

  /// The maximum number of files to enqueue at any given time.
  ///
  /// Set to 0 to use an unbounded channel.
  #[arg(long = "buffer", default_value_t = 256)]
  channel_bound: usize,
}

impl Args {
  pub fn search_path(&self) -> &Path {
    &self.search_path
  }

  pub async fn cache(&self) -> Result<Option<Cache>> {
    self
      .cache_dirpath
      .as_deref()
      .map(Cache::from_dirpath)
      .into_future()
      .await
      .transpose()
  }

  pub async fn config(&self) -> Result<Config> {
    let config = if let Some(config_path) = &self.config_path {
      Config::from_path(config_path).await?
    } else {
      Config::default()
    };

    if let Some(language) = self.language() {
      config.for_language(language).ok()
    } else {
      config.ok()
    }
  }

  pub fn concurrency(&self) -> Result<NonZero<usize>> {
    match self.concurrency {
      Some(num_workers) => num_workers.ok(),
      None => std::thread::available_parallelism().context("failed to get available parallelism"),
    }
  }

  pub fn channel(&self) -> (Sender, Receiver) {
    if self.channel_bound == 0 {
      crate::channel::unbounded()
    } else {
      crate::channel::bounded(self.channel_bound)
    }
  }

  pub fn delimiter(&self) -> char {
    self.delimiter
  }

  pub fn separator(&self) -> char {
    self.separator
  }

  fn language(&self) -> Option<Language> {
    self.language.or(self.extension.as_ref().and_then(Language::from_extension))
  }
}
