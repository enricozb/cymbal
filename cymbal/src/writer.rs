use std::{fmt::Display, thread::JoinHandle};

use anyhow::Context;
use crossbeam::channel::Sender;

use crate::{config::Language, ext::ResultExt, symbol::Symbol};

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
        print!("{symbol}");
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

  pub fn send<P, T>(&self, language: Language, symbol: &Symbol<P, T>) -> Result<(), anyhow::Error>
  where
    P: Display,
    T: Display,
  {
    self
      .sink
      .send(Message::Symbol(format!(
        "{lang}{dlm}{kind}{dlm}{path}{dlm}{line}{dlm}{col}{dlm}{lead}{dlm}{text}{dlm}{tail}{end}",
        lang = language.colored_abbreviation(),
        kind = symbol.kind.colored_abbreviation(),
        path = symbol.path,
        line = symbol.span.start.line,
        col = symbol.span.start.column,
        lead = symbol.lead,
        text = symbol.text,
        tail = symbol.tail,
        dlm = self.delimiter,
        end = self.separator,
      )))
      .context("failed to send message")
  }

  pub fn stop(&self) -> Result<(), anyhow::Error> {
    self.sink.send(Message::Stop).context("failed to send stop message")
  }
}
