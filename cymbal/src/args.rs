use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;

use crate::{
  cache::Cache,
  config::{raw::RawConfig, loader::ConfigLoader, Config, Language},
  ext::{IntoExt, OptionExt, ResultExt},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  /// Language configurations.
  ///
  /// This can either be a path to a .toml file or a TOML string.
  ///
  /// Configs are merged in this order:
  /// 1. Default embedded config (lowest precedence)
  /// 2. Global config (~/.config/cymbal/config.toml or equivalent)
  /// 3. User home config (~/.cymbal.toml)
  /// 4. Project config (.cymbal.toml or .cymbal/config.toml, searched upward)
  /// 5. This explicit config argument (highest precedence)
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
  /// Show detected config file paths and exit.
  #[arg(long)]
  pub show_config_paths: bool,
}

impl Args {
  fn raw_config(&self) -> Result<RawConfig, anyhow::Error> {
    let loader = ConfigLoader::new();

    let user_specified = self.config.as_deref().filter(|s| !s.is_empty());

    loader.load_merged_config(user_specified)
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

  pub fn show_config_paths(&self) -> Result<(), anyhow::Error> {
    let loader = ConfigLoader::new();
    let paths = loader.get_config_paths();

    println!("Config resolution order (lowest to highest precedence):");
    println!("1. Default embedded config: [built-in]");

    if let Some(global) = &paths.global_config {
      let status = if global.exists() { "✓ exists" } else { "✗ not found" };
      println!("2. Global config: {} ({})", global.display(), status);
    } else {
      println!("2. Global config: [not detected]");
    }

    if let Some(user) = &paths.user_config {
      let status = if user.exists() { "✓ exists" } else { "✗ not found" };
      println!("3. User config: {} ({})", user.display(), status);
    } else {
      println!("3. User config: [not detected]");
    }

    if let Some(project) = &paths.project_config {
      let status = if project.exists() { "✓ exists" } else { "✗ not found" };
      println!("4. Project config: {} ({})", project.display(), status);
    } else {
      println!("4. Project config: [not found]");
    }

    if let Some(user_specified) = &self.config {
      if !user_specified.is_empty() {
        println!("5. User-specified config: {} (via --config)", user_specified);
      }
    } else {
      println!("5. User-specified config: [not provided]");
    }

    println!("\nActive config files (will be merged in order):");
    for path in paths.iter_existing() {
      println!("  {}", path.display());
    }

    if let Some(user_specified) = &self.config {
      if !user_specified.is_empty() {
        println!("  {} (user-specified)", user_specified);
      }
    }

    Ok(())
  }
}
