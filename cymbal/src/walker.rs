use std::{
  ffi::OsString,
  io::{BufRead, BufReader},
  os::unix::ffi::OsStringExt,
  path::PathBuf,
  process::{Command, Stdio},
};

use anyhow::Context;
use crossbeam::channel::Receiver;

use crate::ext::ResultExt;

/// A file walker that emits files through a channel.
pub struct Walker {
  pub files: Receiver<PathBuf>,
}

impl Walker {
  pub fn single<P: Into<PathBuf>>(file: P) -> Result<Self, anyhow::Error> {
    let (send, recv) = crossbeam::channel::unbounded();

    send.send(file.into()).context("failed to send")?;

    Self { files: recv }.ok()
  }

  pub fn spawn<'a>(extensions: impl IntoIterator<Item = &'a str>, capacity: usize) -> Result<Self, anyhow::Error> {
    let extension_args: Vec<&str> = extensions.into_iter().flat_map(|ext| vec!["-e", ext]).collect();

    let mut child = Command::new("fd")
      .args(["-t", "f", "-0"])
      .args(extension_args)
      .stdout(Stdio::piped())
      .spawn()
      .context("failed to spawn fd")?;

    let (send, recv) = crossbeam::channel::bounded(capacity);

    let stdout = child.stdout.take().context("stdout")?;

    std::thread::spawn(move || -> Result<(), anyhow::Error> {
      for line in BufReader::new(stdout).split(b'\0') {
        if let Ok(line) = line.map(OsString::from_vec).map(PathBuf::from) {
          send.send(line).context("failed to send")?;
        };
      }

      Ok(())
    });

    Self { files: recv }.ok()
  }
}
