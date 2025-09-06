use std::path::Path;

use anyhow::Result;
use tokio::task::JoinSet;
use walkdir::{DirEntry, WalkDir};

use crate::cache::Cache;
use crate::worker::Worker;

pub struct Walker<'a> {
  cache: Cache,
  path: &'a Path,
}

impl<'a> Walker<'a> {
  pub fn new(cache: Cache, path: &'a Path) -> Self {
    Self { cache, path }
  }

  pub async fn run(&self) {
    let mut tasks = JoinSet::new();

    let walker = WalkDir::new(self.path)
      .into_iter()
      .filter_map(Result::ok)
      .map(DirEntry::into_path)
      .filter(|path| path.is_file());

    for file in walker {
      let cache = self.cache.clone();

      tasks.spawn(async move { Worker::new(file, cache).run().await });
    }

    tasks.join_all().await;
  }
}
