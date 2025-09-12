use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use futures::future::Either;

use crate::cache::Cache;
use crate::config::Config;
use crate::ext::{IntoExt, ResultExt};
use crate::parser::Parser;
use crate::symbol::FileInfo;

pub struct Worker {
  file_path: PathBuf,
  cache: Cache,
  config: &'static Config,
}

enum SymbolStream {
  FromCache,
  FromFile,
}

impl Worker {
  pub fn new(file_path: PathBuf, cache: Cache, config: &'static Config) -> Self {
    Self { file_path, cache, config }
  }

  pub async fn run(&self) -> Result<()> {
    // TODO: try using tokio::fs::metadata
    let file_modified = self.file_path.metadata()?.modified()?.convert::<DateTime<Utc>>();
    let cache_file_info = self.cache.get_file_info(&self.file_path).await?;
    let cache_file_modified = cache_file_info.map(|file_info| file_info.modified);

    // one of the Either does not correctly implement the trait for some reason.
    // yea
    // Stream<Item = Result<Symbol>>

    let symbol_stream = if cache_file_modified.is_none_or(|cache_file_modified| cache_file_modified != file_modified) {
      let file_info = FileInfo::new(&self.file_path, file_modified);
      let Some(parser) = Parser::new(&self.file_path, self.config) else { return ().ok() };

      self.cache.insert_file_info(file_info).await?;

      Either::Left(parser.symbols().await?.map(ResultExt::ok::<anyhow::Error>))
    } else {
      Either::Right(
        self
          .cache
          .symbols(&self.file_path)
          .map(|symbol| symbol.context("failed to fetch symbol")),
      )
    };

    futures::pin_mut!(symbol_stream);

    // causes the `next` call to fail to compile b/c Either only impls Stream if each side does with the same Item
    while let Some(symbol) = symbol_stream.next().await {
      if let Ok(symbol) = symbol {
        println!("{}", symbol.content);
      }
    }

    self.cache.set_is_fully_parsed(&self.file_path).await?;

    ().ok()
  }
}
