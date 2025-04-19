use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use anyhow::Context;
use clap::Parser;

use crate::{
  cache::Cache,
  config::{raw::RawConfig, Config, Language},
  ext::{IntoExt, OptionExt, ResultExt},
};

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
  config: Option<String>,
  /// Directory to cache parsed symbols.
  ///
  /// Files are reparsed if their cached mtime differs from than their current
  /// mtime, or the path of the file doesn't exist in the cache. This option
  /// is typically used when `symbols` is called from the same directory
  /// multiple times, such as searching over a code base in an editor.
  ///
  /// The directory is created if it does not exist.
  #[arg(long)]
  cache_dir: Option<PathBuf>,
  /// The characters between properties of a single symbol.
  ///
  /// This is the character between the path, location, kind, text, and
  /// leading/trailing text written to stdout.
  ///
  /// This defaults to U+200B (zero-width space).
  #[arg(short, long, default_value_t = '\u{200B}')]
  pub delimiter: char,
  /// The character between symbols.
  ///
  /// This defaults to the U+0 (null byte).
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
  /// and otherwise defaults to 8.
  #[arg(short, long)]
  threads: Option<usize>,
  /// Only show symbols from files with extensions matching this language.
  ///
  /// This option takes precedence over `--extension`.
  #[arg(short, long)]
  language: Option<Language>,
  /// Only show symbols from files with extensions matching this extension's
  /// language. Note that this will not filter for symbols in files matching
  /// this extension, but for files with the same language as this extension's.
  ///
  /// The `--language` option takes precedence over `--extension`.
  #[arg(short, long)]
  extension: Option<String>,
  /// The file or directory to search for symbols in.
  ///
  /// If this is a directory, it is recursively searched for files with
  /// supported extensions.
  ///
  /// If this is a file, it is searched for symbols, and `--language` and
  /// `--extension` are ignored, and the langauge appropriate for the file
  /// is used.
  #[arg(default_value = ".")]
  path: Option<PathBuf>,
  /// Print errors to standard error.
  #[arg(long, default_value_t)]
  pub debug: bool,
}

impl Args {
  fn raw_config(&self) -> Result<RawConfig, anyhow::Error> {
    if let Some(config) = &self.config {
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

    RawConfig::default().ok()
  }

  fn language(&self) -> Result<Option<Language>, anyhow::Error> {
    if let Some(language) = self.language {
      return language.some().ok();
    }

    if let Some(extension) = &self.extension {
      return Language::from_extension(extension)
        .context("unknown language")
        .map(Some);
    }

    None.ok()
  }

  pub fn config(&self) -> Result<Config, anyhow::Error> {
    let mut raw_config = self.raw_config()?;

    if let Some(language) = self.language()? {
      raw_config = raw_config.for_language(language);
    }

    raw_config.convert::<Config>().ok()
  }

  pub fn file(&self) -> Option<&Path> {
    if let Some(path) = &self.path {
      if path.is_file() {
        return path.as_path().some();
      }
    }

    None
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
