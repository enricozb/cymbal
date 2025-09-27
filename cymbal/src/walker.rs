use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::Result;
use ignore::Walk;
use tokio::task::JoinHandle;

use crate::cache::Cache;
use crate::channel::{FileTask, Sender};
use crate::config::Language;
use crate::ext::IntoExt;

pub struct Walker {
  path: PathBuf,
  sender: Sender,
  cache: Option<Cache>,
}

impl Walker {
  pub fn new(path: PathBuf, sender: Sender, cache: Option<Cache>) -> Self {
    Self { path, sender, cache }
  }

  pub fn spawn(self) -> JoinHandle<Result<()>> {
    tokio::spawn(self.run())
  }

  async fn run(self) -> Result<()> {
    let walker = Walk::new(self.path).filter_map(Result::ok).filter_map(|dir_entry| {
      let metadata = dir_entry.metadata().ok()?;
      if !metadata.is_file() {
        return None;
      }
      let file_modified = metadata.modified().ok()?;
      let file_path = dir_entry.into_path();
      let language = Language::from_file_path(&file_path)?;

      (file_path, file_modified, language).some()
    });

    let mut file_paths = HashSet::new();

    for (file_path, file_modified, language) in walker {
      let file_task = FileTask::new(file_path.clone(), file_modified.into(), language);

      self.sender.send(file_task).await?;
      file_paths.insert(file_path.clone());
    }

    if let Some(cache) = &self.cache {
      cache.delete_stale_file_paths(file_paths).await?;
    }

    ().ok()
  }
}
