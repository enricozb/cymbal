use std::path::PathBuf;

use anyhow::Result;
use ignore::{DirEntry, Walk};
use tokio::task::JoinHandle;

use crate::channel::Sender;
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
    let walker = Walk::new(self.path)
      .filter_map(Result::ok)
      .map(DirEntry::into_path)
      .filter(|path| path.is_file())
      .filter_map(|file_path| Language::from_file_path(&file_path).map(|language| (file_path, language)));

    for (file_path, language) in walker {
      self.sender.send((file_path, language)).await?;
    }

    ().ok()
  }
}
