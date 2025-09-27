use std::path::PathBuf;

use anyhow::Result;
use ignore::Walk;
use tokio::task::JoinHandle;

use crate::channel::{FileTask, Sender};
use crate::config::Language;
use crate::ext::IntoExt;

pub struct Walker {
  path: PathBuf,
  sender: Sender,
}

impl Walker {
  pub fn new(path: PathBuf, sender: Sender) -> Self {
    Self { path, sender }
  }

  pub fn spawn(self) -> JoinHandle<Result<()>> {
    tokio::spawn(self.run())
  }

  async fn run(self) -> Result<()> {
    let walker = Walk::new(self.path).filter_map(Result::ok).filter_map(|dir_entry| {
      let metadata = dir_entry.metadata().ok()?;
      if !metadata.is_file() {
        return None;
      };
      let file_modified = metadata.modified().ok()?;
      let file_path = dir_entry.into_path();
      let language = Language::from_file_path(&file_path)?;

      (file_path, file_modified, language).some()
    });

    for (file_path, file_modified, language) in walker {
      let file_task = FileTask::new(file_path, file_modified.into(), language);

      self.sender.send(file_task).await?;
    }

    ().ok()
  }
}
