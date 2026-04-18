use std::{collections::HashSet, io::Write, path::Path};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures::{Stream, StreamExt};

use crate::{
  cache::Cache,
  channel::{FileTask, Receiver},
  config::Config,
  ext::{IntoExt, TryStreamExt},
  parser::Parser,
  symbol::Symbol,
};

pub struct Worker<W: Write> {
  cache: Option<Cache>,
  config: &'static Config,
  receiver: Receiver,
  delimiter: char,
  separator: char,
  writer: W,
}

impl<W: Write> Worker<W> {
  pub fn new(cache: Option<Cache>, config: &'static Config, receiver: Receiver, delimiter: char, separator: char, writer: W) -> Self {
    Self {
      cache,
      config,
      receiver,
      delimiter,
      separator,
      writer,
    }
  }

  pub fn into_writer(self) -> W {
    self.writer
  }

  async fn process_file_task(&mut self, file_task: FileTask) -> Result<()> {
    let FileTask {
      ref file_path,
      ref file_modified,
      language,
    } = file_task;

    if !self.config.contains_language(language) {
      return ().ok();
    }

    let Some(cache) = self.cache.take() else {
      let symbol_stream = Parser::new(file_path, language, self.config).symbol_stream().await?;
      self.emit_symbols(file_path, symbol_stream).await?;

      return ().ok();
    };

    if cache.is_file_cached(file_path, file_modified).await? {
      let symbol_stream = cache.get_symbols(file_path).filter_ok();

      return self.emit_symbols(file_path, symbol_stream).await;
    }
    let symbol_stream = Parser::new(file_path, language, self.config).symbol_stream().await?;
    self.cache_and_emit_symbols(&cache, file_path, file_modified, symbol_stream).await?;

    self.cache = Some(cache);

    ().ok()
  }

  pub async fn emit_symbols(&mut self, file_path: &Path, symbol_stream: impl Stream<Item = Symbol>) -> Result<()> {
    let stream = symbol_stream.unique_symbols();
    futures::pin_mut!(stream);
    while let Some(symbol) = stream.next().await {
      self.write_symbol(file_path, &symbol)?;
    }

    ().ok()
  }

  async fn cache_and_emit_symbols(
    &mut self,
    cache: &Cache,
    file_path: &Path,
    file_modified: &DateTime<Utc>,
    symbol_stream: impl Stream<Item = Symbol>,
  ) -> Result<()> {
    let symbol_stream = symbol_stream.unique_symbols();
    let mut symbols = Vec::new();
    futures::pin_mut!(symbol_stream);

    while let Some(symbol) = symbol_stream.next().await {
      self.write_symbol(file_path, &symbol)?;

      symbols.push(symbol);
    }

    cache.insert_file(file_path, file_modified).await?;
    cache.insert_symbols(file_path, &symbols).await?;
    cache.set_file_is_fully_parsed(file_path).await?;

    ().ok()
  }

  pub fn write_symbol(&mut self, file_path: &Path, symbol: &Symbol) -> Result<()> {
    write!(
      self.writer,
      "{lang}{dlm}{kind}{dlm}{path}{dlm}{line}{dlm}{col}{dlm}{lead}{dlm}{text}{dlm}{trail}{end}",
      lang = symbol.language.colored(),
      kind = symbol.kind.colored(),
      path = file_path.display(),
      line = symbol.line,
      col = symbol.column,
      lead = symbol.leading_str(),
      text = symbol.content,
      trail = symbol.trailing_str(),
      dlm = self.delimiter,
      end = self.separator,
    )
    .context("failed to write symbol")
  }

  pub async fn run(mut self) -> Result<()> {
    // TODO(enricozb): try using `self.receiver` as a `Stream`.
    while let Ok(file_task) = self.receiver.recv().await {
      self.process_file_task(file_task).await?;
    }

    ().ok()
  }
}

#[extend::ext]
impl<T: Stream<Item = Symbol>> T {
  fn unique_symbols(self) -> impl Stream<Item = Symbol> {
    let mut symbol_positions = HashSet::<(i64, i64)>::new();

    // TODO(enricozb): it looks like some symbols are appearing before others,
    // and precedence is not respected. Specifically in the rust language it
    // seems that methods, despite appearing before functions in the default
    // config, do not override the functions in the outputted symbols.
    self.filter_map(move |symbol| {
      let position = (symbol.line, symbol.column);
      if symbol_positions.contains(&position) {
        None.ready()
      } else {
        symbol_positions.insert(position);
        symbol.some().ready()
      }
    })
  }
}
