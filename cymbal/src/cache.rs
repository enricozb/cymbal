use std::{
  collections::HashMap,
  fs::File,
  path::{Path, PathBuf},
  time::SystemTime,
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{ext::ResultExt, symbol::Symbol};

const CACHE_FILE_NAME: &str = "cache.json";

#[derive(Default)]
pub struct Cache {
  path: Option<PathBuf>,
  files: HashMap<PathBuf, FileInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct FileInfo {
  pub modified: SystemTime,
  /// Cached symbols don't contain their own paths  as they are stored in the
  /// [`Cache::files`] field.
  pub symbols: Vec<Symbol<(), String>>,
}

impl Cache {
  /// Read a cache from a directory containing the cache.
  ///
  /// If the directory does not exist or does not contain the cache file,
  /// the directory and file are created, and a default cache is returned.
  pub fn from_dir<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
    let path = path.as_ref().join(CACHE_FILE_NAME);

    if !path.exists() {
      std::fs::create_dir_all(path.parent().context("parent")?).context("create dir")?;

      return Ok(Self {
        path: Some(path.clone()),
        files: HashMap::default(),
      });
    }

    let file = File::open(&path).context("open")?;

    Ok(Self {
      path: Some(path.clone()),
      files: serde_json::from_reader(file)
        .context("failed to parse cache")
        .warn()
        .unwrap_or_default(),
    })
  }

  /// Returns the [`FileInfo`] for file at a given path, if any.
  pub fn get_file_info(&self, path: &PathBuf) -> Option<&FileInfo> {
    self.files.get(path)
  }

  /// Inserts a new [`FileInfo`] for a file at a given path.
  pub fn insert_file_info(&mut self, path: PathBuf, modified: SystemTime) {
    self.files.insert(
      path,
      FileInfo {
        modified,
        symbols: Vec::new(),
      },
    );
  }

  /// Inserts a new [`Symbol`] for a file at a given path.
  ///
  /// [`insert_file_info`] must be called first on this path.
  pub fn insert_symbol<P, T: Into<String>>(&mut self, path: &Path, symbol: Symbol<P, T>) -> Result<(), anyhow::Error> {
    self
      .files
      .get_mut(path)
      .with_context(|| format!("inserting symbol into unknown path: {path:?}"))?
      .symbols
      .push(symbol.forget_path());

    Ok(())
  }

  /// Save a cache to its path.
  pub fn save(&self) -> Result<(), anyhow::Error> {
    let Some(path) = &self.path else {
      return Ok(());
    };

    let json = serde_json::to_string(&self.files).context("to_string")?;

    std::fs::write(path, json).context("write")
  }
}
