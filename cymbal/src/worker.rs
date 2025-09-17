use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::StreamExt;

use crate::cache::Cache;
use crate::channel::Receiver;
use crate::config::{Config, Language};
use crate::ext::IntoExt;
use crate::parser::Parser;
use crate::symbol_stream::{CacheSymbolStream, ParserSymbolStream, SymbolStream};

pub struct Worker {
  cache: Option<Cache>,
  config: &'static Config,
  receiver: Receiver,
}

impl Worker {
  pub fn new(cache: Option<Cache>, config: &'static Config, receiver: Receiver) -> Self {
    Self { cache, config, receiver }
  }

  async fn is_cached<'a>(&'a self, file_path: &Path) -> Result<Option<&'a Cache>> {
    let Some(cache) = &self.cache else { return None.ok() };
    let file_modified = file_path.metadata()?.modified()?.convert::<DateTime<Utc>>();
    let cache_file_info = cache.get_file_info(file_path).await?;
    let cache_file_modified = cache_file_info.map(|file_info| file_info.modified);
    let is_cached = cache_file_modified.is_none_or(|cache_file_modified| cache_file_modified != file_modified);

    if is_cached { cache.some().ok() } else { None.ok() }
  }

  async fn symbol_stream(
    &self,
    file_path: &Path,
    language: Language,
  ) -> Result<SymbolStream<impl ParserSymbolStream, impl CacheSymbolStream>> {
    if let Some(cache) = self.is_cached(file_path).await? {
      SymbolStream::FromCache(cache.symbols(file_path))
    } else {
      let parser = Parser::new(file_path, language, self.config);

      SymbolStream::FromParser(parser.symbol_stream().await?)
    }
    .ok()
  }

  pub async fn run(self) -> Result<()> {
    // TODO(enricozb): try using `self.receiver` as a `Stream`.
    while let Ok((file_path, language)) = self.receiver.recv().await {
      let symbol_stream = self.symbol_stream(&file_path, language).await?.into_stream();

      futures::pin_mut!(symbol_stream);

      while let Some(symbol) = symbol_stream.next().await {
        println!("{symbol:?}");
      }
    }

    ().ok()
  }
}
