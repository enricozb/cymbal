use std::{fmt::Display, thread::JoinHandle};

use anyhow::Context;
use crossbeam::channel::Sender;

use crate::{ext::ResultExt, symbol::Symbol};

pub enum Message {
  Symbol(String),
  Stop,
}

#[derive(Clone)]
pub struct Writer {
  separator: char,
  delimiter: char,

  pub sink: Sender<Message>,
}

impl Writer {
  pub fn spawn(separator: char, delimiter: char, capacity: usize) -> Result<(Self, JoinHandle<()>), anyhow::Error> {
    let (send, recv) = crossbeam::channel::bounded(capacity);

    let handle = std::thread::spawn(move || {
      while let Ok(Message::Symbol(symbol)) = recv.recv() {
        print!("{symbol}")
      }
    });

    (
      Self {
        separator,
        delimiter,
        sink: send,
      },
      handle,
    )
      .ok()
  }

  pub fn send<Path, Text>(&self, symbol: Symbol<Path, Text>) -> Result<(), anyhow::Error>
  where
    Path: Display,
    Text: Display,
  {
    self
      .sink
      .send(Message::Symbol(format!(
        "{path}{sep}{line}{sep}{col}{sep}{text}{sep}{kind}{end}",
        path = symbol.path,
        line = symbol.span.start.line,
        col = symbol.span.start.column,
        text = symbol.text,
        kind = symbol.kind.colored_abbreviation(),
        sep = self.separator,
        end = self.delimiter,
      )))
      .context("failed to send message")
  }

  pub fn stop(&self) -> Result<(), anyhow::Error> {
    self.sink.send(Message::Stop).context("failed to send stop message")
  }
}
