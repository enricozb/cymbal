use std::{borrow::Cow, path::PathBuf};

use anyhow::Context;
use clap::Parser;

use crate::{cache::Cache, config::Config};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  /// Language configurations.
  ///
  /// This can either be a path to a .toml file or a TOML string.
  ///
  /// The default configuration will be applied if this argument is not
  /// provided or if it is the empty string.
  #[arg(short, long)]
  language_config: Option<String>,
  /// Directory to cache parsed symbols.
  ///
  /// Files are reparsed if their cached mtime differs from than their current
  /// mtime. The cache is only usable if the previously generated relative
  /// paths are still valid. This would normally only be the case when the
  /// binary is called from the same directory multiple times.
  ///
  /// This directory is created if it does not exist.
  #[arg(short, long)]
  cache_dir: Option<PathBuf>,
  /// The characters between properties of a single symbol.
  ///
  /// This is the character between the path, location, kind, text, and
  /// leading/trailing text written to stdout.
  #[arg(short, long, default_value_t = '\u{2008}')]
  pub delimiter: char,
  /// The character between symbols.
  #[arg(short, long, default_value_t = '\0')]
  pub separator: char,
  /// Whether to spawn a detached process to index symbols.
  ///
  /// Only useful with the `cache-dir` option.
  ///
  /// If this option is false, the cache may not be created if the process
  /// is exited prematurely. This can happen if using `symbols` in a pipeline
  /// (such as with `fzf`) and selecting a symbol before indexing is complete.
  ///
  /// If this option is true, indexing is performed by a separate detached
  /// process whose output is redirected to stdout. Then, if `symbols` is
  /// exited prematurely, the indexing will still be able to complete.
  #[arg(long, default_value_t)]
  detached: bool,
  /// The number of worker threads to use when parsing files.
  ///
  /// This defaults to `std::thread::available_parallelism` if it is available,
  /// and otherwise is 8.
  #[arg(short, long)]
  threads: Option<usize>,
}

impl Args {
  pub fn config(&self) -> Result<Config, anyhow::Error> {
    if let Some(config) = &self.language_config {
      let config_content: Cow<str> = if config.ends_with(".toml") {
        std::fs::read_to_string(config)
          .context("failed to read config file")?
          .into()
      } else {
        config.into()
      };

      if !config_content.is_empty() {
        return toml::from_str(&config_content).context("failed to parse config");
      }
    }

    Ok(Config::default())
  }

  pub fn cache(&self) -> Result<Cache, anyhow::Error> {
    if let Some(cache_dir) = &self.cache_dir {
      Cache::from_dir(cache_dir)
    } else {
      Ok(Cache::default())
    }
  }

  pub fn num_threads(&self) -> usize {
    self
      .threads
      .or_else(|| std::thread::available_parallelism().map(usize::from).ok())
      .unwrap_or(8)
  }
}
