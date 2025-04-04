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
  delimiter: char,
  separator: char,

  pub sink: Sender<Message>,
}

impl Writer {
  pub fn spawn(delimiter: char, separator: char, capacity: usize) -> Result<(Self, JoinHandle<()>), anyhow::Error> {
    let (send, recv) = crossbeam::channel::bounded(capacity);

    let handle = std::thread::spawn(move || {
      while let Ok(Message::Symbol(symbol)) = recv.recv() {
        print!("{symbol}")
      }
    });

    (
      Self {
        delimiter,
        separator,
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
        "{path}{dlm}{line}{dlm}{col}{dlm}{lead}{dlm}{text}{dlm}{tail}{dlm}{kind}{end}",
        path = symbol.path,
        line = symbol.span.start.line,
        col = symbol.span.start.column,
        lead = symbol.lead,
        text = symbol.text,
        tail = symbol.tail,
        kind = symbol.kind.colored_abbreviation(),
        dlm = self.delimiter,
        end = self.separator,
      )))
      .context("failed to send message")
  }

  pub fn stop(&self) -> Result<(), anyhow::Error> {
    self.sink.send(Message::Stop).context("failed to send stop message")
  }
}
