use std::path::Path;

use anyhow::Result;
use tokio::task::JoinSet;
use walkdir::{DirEntry, WalkDir};

use crate::cache::Cache;
use crate::config::Config;
use crate::ext::Leak;
use crate::worker::Worker;

pub struct Walker<'a> {
  path: &'a Path,
  cache: Cache,
  config: Config,
}

impl<'a> Walker<'a> {
  pub fn new(path: &'a Path, cache: Cache, config: Config) -> Self {
    Self { path, cache, config }
  }

  pub async fn run(self) {
    let config = self.config.leak();
    let mut tasks = JoinSet::new();
    let walker = WalkDir::new(self.path)
      .into_iter()
      .filter_map(Result::ok)
      .map(DirEntry::into_path)
      .filter(|path| path.is_file());

    for file in walker {
      let cache = self.cache.clone();

      tasks.spawn(async move { Worker::new(file, cache, config).run().await });
    }

    tasks.join_all().await;
  }
}
