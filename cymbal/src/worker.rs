use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::{Stream, StreamExt};

use crate::cache::Cache;
use crate::channel::{FileTask, Receiver};
use crate::config::Config;
use crate::ext::{IntoExt, TryStreamExt};
use crate::parser::Parser;
use crate::symbol::Symbol;

pub struct Worker {
  cache: Option<Cache>,
  config: &'static Config,
  receiver: Receiver,
  delimiter: char,
  separator: char,
}

impl Worker {
  pub fn new(cache: Option<Cache>, config: &'static Config, receiver: Receiver, delimiter: char, separator: char) -> Self {
    Self {
      cache,
      config,
      receiver,
      delimiter,
      separator,
    }
  }

  async fn process_file_task(&self, file_task: FileTask) -> Result<()> {
    let FileTask {
      file_path,
      file_modified,
      language,
    } = &file_task;

    let Some(cache) = &self.cache else {
      let symbol_stream = Parser::new(file_path, *language, self.config).symbol_stream().await?;
      self.emit_symbols(file_path, symbol_stream).await;

      return ().ok();
    };

    if cache.is_file_cached(file_path, file_modified).await? {
      let symbol_stream = cache.get_symbols(file_path).filter_ok();
      self.emit_symbols(file_path, symbol_stream).await;

      return ().ok();
    }

    let symbol_stream = Parser::new(file_path, *language, self.config).symbol_stream().await?;

    self.cache_and_emit_symbols(cache, file_path, file_modified, symbol_stream).await
  }

  async fn emit_symbols(&self, file_path: &Path, symbol_stream: impl Stream<Item = Symbol>) {
    symbol_stream
      .map(|symbol| self.print_symbol(file_path, &symbol))
      .collect::<()>()
      .await;
  }

  async fn cache_and_emit_symbols(
    &self,
    cache: &Cache,
    file_path: &Path,
    file_modified: &DateTime<Utc>,
    symbol_stream: impl Stream<Item = Symbol>,
  ) -> Result<()> {
    let mut symbols = Vec::new();
    futures::pin_mut!(symbol_stream);

    while let Some(symbol) = symbol_stream.next().await {
      self.print_symbol(file_path, &symbol);

      symbols.push(symbol);
    }

    cache.insert_file(file_path, file_modified).await?;
    cache.insert_symbols(file_path, &symbols).await?;
    cache.set_file_is_fully_parsed(file_path).await?;

    ().ok()
  }

  fn print_symbol(&self, file_path: &Path, symbol: &Symbol) {
    print!(
      "{lang}{dlm}{kind}{dlm}{path}{dlm}{line}{dlm}{col}{dlm}{lead}{dlm}{text}{dlm}{trail}{end}",
      lang = symbol.language.colored_abbreviation(),
      kind = symbol.kind.colored_abbreviation(),
      path = file_path.display(),
      line = symbol.line,
      col = symbol.column,
      lead = symbol.leading_str(),
      text = symbol.content,
      trail = symbol.trailing_str(),
      dlm = self.delimiter,
      end = self.separator,
    );
  }

  pub async fn run(self) -> Result<()> {
    // TODO(enricozb): try using `self.receiver` as a `Stream`.
    while let Ok(file_task) = self.receiver.recv().await {
      self.process_file_task(file_task).await?;
    }

    ().ok()
  }
}
