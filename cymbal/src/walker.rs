use std::path::Path;

use anyhow::Result;
use ignore::{DirEntry, Walk};
use tokio::task::JoinSet;

use crate::cache::Cache;
use crate::config::{Config, Language};
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
    let walker = Walk::new(self.path)
      .filter_map(Result::ok)
      .map(DirEntry::into_path)
      .filter(|path| path.is_file())
      .filter_map(|file_path| Language::from_file_path(&file_path).map(|language| (file_path, language)));

    for (file_path, language) in walker {
      let cache = self.cache.clone();

      tasks.spawn(async move {
        if let Err(err) = Worker::new(file_path, language, cache, config).run().await {
          println!("{err:?}");
        }
      });
    }

    tasks.join_all().await;
  }
}
