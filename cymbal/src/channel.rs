use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::config::Language;

pub struct FileTask {
  pub file_path: PathBuf,
  pub file_modified: DateTime<Utc>,
  pub language: Language,
}

impl FileTask {
  pub fn new(file_path: PathBuf, file_modified: DateTime<Utc>, language: Language) -> Self {
    Self {
      file_path,
      file_modified,
      language,
    }
  }
}

pub type Sender = async_channel::Sender<FileTask>;
pub type Receiver = async_channel::Receiver<FileTask>;

pub fn bounded(cap: usize) -> (Sender, Receiver) {
  async_channel::bounded(cap)
}

pub fn unbounded() -> (Sender, Receiver) {
  async_channel::unbounded()
}
