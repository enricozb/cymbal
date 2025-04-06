use std::{path::PathBuf, sync::Arc, thread::JoinHandle, time::SystemTime};

use anyhow::Context;
use crossbeam::channel::Receiver;
use parking_lot::RwLock;

use crate::{cache::Cache, config::Config, parser::Parser, symbol::Symbol, writer::Writer};

pub struct Worker {
  config: &'static Config,
  cache: Arc<RwLock<Cache>>,
  files: Receiver<PathBuf>,
  writer: Writer,
}

impl Worker {
  pub fn new(config: &'static Config, cache: Arc<RwLock<Cache>>, files: Receiver<PathBuf>, writer: Writer) -> Self {
    Self {
      config,
      cache,
      files,
      writer,
    }
  }

  pub fn spawn(self) -> JoinHandle<Result<(), anyhow::Error>> {
    std::thread::spawn(move || {
      while let Ok(path) = self.files.recv() {
        let modified = std::fs::metadata(&path)
          .and_then(|m| m.modified())
          .with_context(|| format!("failed to get metadata for {path:?}"))?;

        if self.use_cached_entries(&path, modified)? {
          continue;
        }

        self.parse_file(&path, modified)?;
      }

      Ok(())
    })
  }

  /// Attempts to use the cache to compute a paths entries.
  ///
  /// Returns true if the cache's entries were used.
  fn use_cached_entries(&self, path: &PathBuf, modified: SystemTime) -> Result<bool, anyhow::Error> {
    if let Some(file_info) = self.cache.read().get_file_info(path) {
      // if the cached file and the current file have the same modified timestamp,
      // use the entries from the cache.
      if modified == file_info.modified {
        for symbol in &file_info.symbols {
          // cached entries don't contain paths so they are re-inserted here.
          self
            .writer
            .send(Symbol {
              path: path.as_os_str().to_string_lossy(),
              span: symbol.span,
              lead: &symbol.lead,
              text: &symbol.text,
              tail: &symbol.tail,
              kind: symbol.kind,
            })
            .context("failed to send symbol to writer")?;
        }

        return Ok(true);
      }
    }

    Ok(false)
  }

  /// Parses a file and inserts its entries into the cache.
  fn parse_file(&self, path: &PathBuf, modified: SystemTime) -> Result<(), anyhow::Error> {
    self.cache.write().insert_file_info(path.clone(), modified);

    let Some(parser) = Parser::from_path(self.config, path) else {
      return Ok(());
    };

    parser.on_symbol(|symbol| {
      self
        .writer
        .send(Symbol {
          path: path.as_os_str().to_string_lossy(),
          span: symbol.span,
          lead: symbol.lead,
          text: symbol.text,
          tail: symbol.tail,
          kind: symbol.kind,
        })
        .context("failed to send symbol")?;

      self
        .cache
        .write()
        .insert_symbol(path, symbol)
        .context("failed to insert symbol into cache")?;

      Ok(())
    })
  }
}
