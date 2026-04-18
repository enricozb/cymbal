#![feature(lazy_cell_into_inner)]

mod args;
mod walker;

use anyhow::Result;
use clap::Parser;
use cymbal::{cache, channel, config, ext, worker};
use tokio::task::JoinSet;

use crate::{
  args::Args,
  ext::{IntoExt, IteratorExt, Leak},
  walker::Walker,
  worker::Worker,
};

#[tokio::main]
async fn main() -> Result<()> {
  let args = Args::parse();
  let available_concurrency = args.concurrency()?.convert::<usize>();
  let cache = args.cache().await?;
  let config = args.config().await?.leak();
  let (sender, receiver) = args.channel();
  let delimiter = args.delimiter();
  let separator = args.separator();
  let should_clean_cache = !args.is_filtering();
  let walker = Walker::new(args.search_path().to_path_buf(), sender, cache.clone(), should_clean_cache).spawn();

  let mut workers = JoinSet::new();
  for _ in 0..available_concurrency {
    workers.spawn(Worker::new(cache.clone(), config, receiver.clone(), delimiter, separator, std::io::stdout()).run());
  }
  workers.join_all().await.ok_all()?;

  walker.await?
}
