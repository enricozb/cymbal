use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::cache::Cache;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  /// Directory to cache parsed symbols.
  ///
  /// Files are reparsed if their cached mtime differs from than their current
  /// mtime, or the path of the file doesn't exist in the cache. This option
  /// is typically used when `symbols` is called from the same directory
  /// multiple times, such as searching over a code base in an editor.
  #[arg(long)]
  pub cache_dirpath: Option<PathBuf>,
  /// The file or directory to search for symbols in.
  ///
  /// If this is a directory, it is recursively searched for files with
  /// supported extensions.
  ///
  /// If this is a file, it is searched for symbols, and `--language` and
  /// `--extension` are ignored, and the langauge appropriate for the file
  /// is used.
  #[arg(default_value = ".")]
  pub path: Option<PathBuf>,
}

impl Args {
  pub async fn cache(&self) -> Result<Cache> {
    if let Some(cache_dirpath) = &self.cache_dirpath {
      Cache::from_dirpath(cache_dirpath).await
    } else {
      Cache::new().await
    }
  }
}
