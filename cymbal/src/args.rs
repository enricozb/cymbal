use std::num::NonZero;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use crate::cache::Cache;
use crate::channel::{Receiver, Sender};
use crate::config::Config;
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

  /// Directory to cache parsed symbols.
  ///
  /// Files are reparsed if their cached mtime differs from than their current
  /// mtime, or the path of the file doesn't exist in the cache. This option
  /// is typically used when `symbols` is called from the same directory
  /// multiple times, such as searching over a code base in an editor.
  #[arg(long = "cache")]
  cache_dirpath: Option<PathBuf>,

  /// The file or directory to search for symbols in.
  ///
  /// If this is a directory, it is recursively searched for files with
  /// supported extensions.
  ///
  /// If this is a file, it is searched for symbols, and `--language` and
  /// `--extension` are ignored, and the langauge appropriate for the file
  /// is used.
  #[arg(default_value = ".")]
  pub search_path: PathBuf,

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
    if let Some(config_path) = &self.config_path {
      Config::from_path(config_path).await
    } else {
      Config::default().ok()
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
}
