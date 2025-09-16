use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use tokio::task::JoinHandle;

use crate::cache::Cache;
use crate::channel::Receiver;
use crate::config::{Config, Language};
use crate::ext::{IntoExt, ResultExt};
use crate::parser::Parser;
use crate::symbol_stream::{CacheSymbolStream, ParserSymbolStream, SymbolStream};

pub struct Worker {
  cache: Cache,
  config: &'static Config,
  receiver: Receiver,
}

impl Worker {
  pub fn new(cache: Cache, config: &'static Config, receiver: Receiver) -> Self {
    Self { cache, config, receiver }
  }

  pub fn spawn(self) -> JoinHandle<Result<()>> {
    tokio::spawn(self.run())
  }

  async fn symbol_stream(
    &self,
    file_path: &Path,
    language: Language,
  ) -> Result<SymbolStream<impl ParserSymbolStream, impl CacheSymbolStream>> {
    let file_modified = file_path.metadata()?.modified()?.convert::<DateTime<Utc>>();
    let cache_file_info = self.cache.get_file_info(file_path).await?;
    let cache_file_modified = cache_file_info.map(|file_info| file_info.modified);

    if cache_file_modified.is_none_or(|cache_file_modified| cache_file_modified != file_modified) {
      let parser = Parser::new(file_path, language, self.config);

      SymbolStream::FromParser(parser.symbol_stream().await?)
    } else {
      SymbolStream::FromCache(self.cache.symbols(file_path))
    }
    .ok()
  }

  pub async fn run(self) -> Result<()> {
    // TODO(enricozb): try using `self.receiver` as a `Stream`.
    while let Ok((file_path, language)) = self.receiver.recv().await {
      let symbol_stream = self.symbol_stream(&file_path, language).await?.into_stream();

      futures::pin_mut!(symbol_stream);

      while let Some(symbol) = symbol_stream.next().await {
        println!("{file_path:?} {symbol:?}");
      }
    }

    println!("worker channel closed?");

    ().ok()
  }

  // async fn run(&self) -> Result<()> «…»󠅫︊󠄐󠄐󠄐󠄐󠅜󠅕󠅤󠄐󠅣󠅩󠅝󠅒󠅟󠅜󠅏󠅣󠅤󠅢󠅕󠅑󠅝󠄐󠄭󠄐󠅣󠅕󠅜󠅖󠄞󠅣󠅩󠅝󠅒󠅟󠅜󠅏󠅣󠅤󠅢󠅕󠅑󠅝󠄘󠄙󠄞󠅑󠅧󠅑󠅙󠅤󠄯󠄫︊︊󠄐󠄐󠄐󠄐󠅝󠅑󠅤󠅓󠅘󠄐󠅣󠅩󠅝󠅒󠅟󠅜󠅏󠅣󠅤󠅢󠅕󠅑󠅝󠄐󠅫︊󠄐󠄐󠄐󠄐󠄐󠄐󠅃󠅩󠅝󠅒󠅟󠅜󠅃󠅤󠅢󠅕󠅑󠅝󠄪󠄪󠄶󠅢󠅟󠅝󠅀󠅑󠅢󠅣󠅕󠅢󠄘󠅠󠅑󠅢󠅣󠅕󠅢󠅏󠅣󠅩󠅝󠅒󠅟󠅜󠅏󠅣󠅤󠅢󠅕󠅑󠅝󠄙󠄐󠄭󠄮󠄐󠅫︊󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠅖󠅥󠅤󠅥󠅢󠅕󠅣󠄪󠄪󠅠󠅙󠅞󠅏󠅝󠅥󠅤󠄑󠄘󠅠󠅑󠅢󠅣󠅕󠅢󠅏󠅣󠅩󠅝󠅒󠅟󠅜󠅏󠅣󠅤󠅢󠅕󠅑󠅝󠄙󠄫︊󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠅧󠅘󠅙󠅜󠅕󠄐󠅜󠅕󠅤󠄐󠅃󠅟󠅝󠅕󠄘󠅣󠅩󠅝󠅒󠅟󠅜󠄙󠄐󠄭󠄐󠅠󠅑󠅢󠅣󠅕󠅢󠅏󠅣󠅩󠅝󠅒󠅟󠅜󠅏󠅣󠅤󠅢󠅕󠅑󠅝󠄞󠅞󠅕󠅨󠅤󠄘󠄙󠄞󠅑󠅧󠅑󠅙󠅤󠄐󠅫︊󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄟󠄟󠄐󠅣󠅕󠅜󠅖󠄞󠅓󠅑󠅓󠅘󠅕󠄞󠅙󠅞󠅣󠅕󠅢󠅤󠅏󠅣󠅩󠅝󠅒󠅟󠅜󠄘󠄖󠅣󠅕󠅜󠅖󠄞󠅖󠅙󠅜󠅕󠅏󠅠󠅑󠅤󠅘󠄜󠄐󠄖󠅣󠅩󠅝󠅒󠅟󠅜󠄙󠄞󠅑󠅧󠅑󠅙󠅤󠄯󠄫︊︊󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠅠󠅢󠅙󠅞󠅤󠅜󠅞󠄑󠄘󠄒󠅫󠅭󠄒󠄜󠄐󠅣󠅩󠅝󠅒󠅟󠅜󠄞󠅓󠅟󠅞󠅤󠅕󠅞󠅤󠄙󠄫︊󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠅭︊︊󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄟󠄟󠄐󠅣󠅕󠅜󠅖󠄞󠅓󠅑󠅓󠅘󠅕󠄞󠅣󠅕󠅤󠅏󠅙󠅣󠅏󠅖󠅥󠅜󠅜󠅩󠅏󠅠󠅑󠅢󠅣󠅕󠅔󠄘󠄖󠅣󠅕󠅜󠅖󠄞󠅖󠅙󠅜󠅕󠅏󠅠󠅑󠅤󠅘󠄙󠄞󠅑󠅧󠅑󠅙󠅤󠄯󠄫︊󠄐󠄐󠄐󠄐󠄐󠄐󠅭︊︊󠄐󠄐󠄐󠄐󠄐󠄐󠅃󠅩󠅝󠅒󠅟󠅜󠅃󠅤󠅢󠅕󠅑󠅝󠄪󠄪󠄶󠅢󠅟󠅝󠄳󠅑󠅓󠅘󠅕󠄘󠅓󠅑󠅓󠅘󠅕󠅏󠅣󠅩󠅝󠅒󠅟󠅜󠅏󠅣󠅤󠅢󠅕󠅑󠅝󠄙󠄐󠄭󠄮󠄐󠅫︊󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠅖󠅥󠅤󠅥󠅢󠅕󠅣󠄪󠄪󠅠󠅙󠅞󠅏󠅝󠅥󠅤󠄑󠄘󠅓󠅑󠅓󠅘󠅕󠅏󠅣󠅩󠅝󠅒󠅟󠅜󠅏󠅣󠅤󠅢󠅕󠅑󠅝󠄙󠄫︊󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠅧󠅘󠅙󠅜󠅕󠄐󠅜󠅕󠅤󠄐󠅃󠅟󠅝󠅕󠄘󠅣󠅩󠅝󠅒󠅟󠅜󠄙󠄐󠄭󠄐󠅓󠅑󠅓󠅘󠅕󠅏󠅣󠅩󠅝󠅒󠅟󠅜󠅏󠅣󠅤󠅢󠅕󠅑󠅝󠄞󠅞󠅕󠅨󠅤󠄘󠄙󠄞󠅑󠅧󠅑󠅙󠅤󠄐󠅫︊󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠅠󠅢󠅙󠅞󠅤󠅜󠅞󠄑󠄘󠄒󠅫󠄪󠄯󠅭󠄒󠄜󠄐󠅣󠅩󠅝󠅒󠅟󠅜󠄞󠅝󠅑󠅠󠄘󠅬󠅣󠅩󠅝󠅒󠅟󠅜󠅬󠄐󠅣󠅩󠅝󠅒󠅟󠅜󠄞󠅓󠅟󠅞󠅤󠅕󠅞󠅤󠄞󠅓󠅜󠅟󠅞󠅕󠄘󠄙󠄙󠄙󠄫︊󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠄐󠅭︊󠄐󠄐󠄐󠄐󠄐󠄐󠅭︊󠄐󠄐󠄐󠄐󠅭︊︊󠄐󠄐󠄐󠄐󠄘󠄙󠄞󠅟󠅛󠄘󠄙︊󠄐󠄐󠅭
}
