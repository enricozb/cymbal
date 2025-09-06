use std::path::PathBuf;

use anyhow::Result;

use crate::cache::Cache;
use crate::ext::ResultExt;

pub struct Worker {
  filepath: PathBuf,
  cache: Cache,
}

impl Worker {
  pub fn new(filepath: PathBuf, cache: Cache) -> Self {
    Self { filepath, cache }
  }

  pub async fn run(&self) -> Result<()> {
    let modified = self.filepath.metadata()?.modified()?;

    println!("worker: {:?}", self.filepath);

    ().ok()
  }
}
