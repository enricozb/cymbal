use std::path::PathBuf;

use crate::config::Language;

pub type FileTask = (PathBuf, Language);
pub type Sender = async_channel::Sender<FileTask>;
pub type Receiver = async_channel::Receiver<FileTask>;

pub fn bounded(cap: usize) -> (Sender, Receiver) {
  async_channel::bounded(cap)
}

pub fn unbounded() -> (Sender, Receiver) {
  async_channel::unbounded()
}
