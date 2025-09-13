use std::path::PathBuf;

use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::StreamExt;

use crate::cache::Cache;
use crate::config::{Config, Language};
use crate::ext::{IntoExt, ResultExt};
use crate::parser::Parser;
use crate::symbol::FileInfo;
use crate::symbol_stream::{CacheSymbolStream, ParserSymbolStream, SymbolStream};

pub struct Worker {
  file_path: PathBuf,
  language: Language,
  cache: Cache,
  config: &'static Config,
}

impl Worker {
  pub fn new(file_path: PathBuf, language: Language, cache: Cache, config: &'static Config) -> Self {
    Self {
      file_path,
      language,
      cache,
      config,
    }
  }

  async fn symbol_stream(&self) -> Result<SymbolStream<impl ParserSymbolStream, impl CacheSymbolStream>> {
    let file_modified = self.file_path.metadata()?.modified()?.convert::<DateTime<Utc>>();
    let cache_file_info = self.cache.get_file_info(&self.file_path).await?;
    let cache_file_modified = cache_file_info.map(|file_info| file_info.modified);

    if cache_file_modified.is_none_or(|cache_file_modified| cache_file_modified != file_modified) {
      let parser = Parser::new(&self.file_path, self.language, self.config);

      SymbolStream::FromParser(parser.symbol_stream().await?)
    } else {
      SymbolStream::FromCache(self.cache.symbols(&self.file_path))
    }
    .ok()
  }

  pub async fn run(&self) -> Result<()> {
    let symbol_stream = self.symbol_stream().await?;

    match symbol_stream {
      SymbolStream::FromParser(parser_symbol_stream) => {
        futures::pin_mut!(parser_symbol_stream);
        while let Some(symbol) = parser_symbol_stream.next().await {
          // self.cache.insert_symbol(&self.file_path, &symbol).await?;

          println!("{}", symbol.content);
        }

        // self.cache.set_is_fully_parsed(&self.file_path).await?;
      }

      SymbolStream::FromCache(cache_symbol_stream) => {
        futures::pin_mut!(cache_symbol_stream);
        while let Some(symbol) = cache_symbol_stream.next().await {
          println!("{:?}", symbol.map(|symbol| symbol.content.clone()));
        }
      }
    }

    ().ok()
  }
}
