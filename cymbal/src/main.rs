#![feature(lazy_cell_into_inner)]

mod args;
mod cache;
mod channel;
mod color;
mod config;
mod ext;
mod parser;
mod symbol;
mod template;
mod utils;
mod walker;
mod worker;

use anyhow::Result;
use clap::Parser;
use tokio::task::JoinSet;

use crate::args::Args;
use crate::ext::{IntoExt, IteratorExt, Leak};
use crate::walker::Walker;
use crate::worker::Worker;

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

  // TODO:
  // - don't clean cache when filtering
  // - dedup symbols at the same start
  // - tree sitter languages and queries must be shared

  let mut workers = JoinSet::new();
  for _ in 0..available_concurrency {
    workers.spawn(Worker::new(cache.clone(), config, receiver.clone(), delimiter, separator).run());
  }
  workers.join_all().await.ok_all()?;

  walker.await?
}
