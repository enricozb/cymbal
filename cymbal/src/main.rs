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
  let walker = Walker::new(args.search_path, sender, cache.clone()).spawn();

  // TODO:
  // - synchronous walker spawning async worker tasks, joining at the end.
  // - cache must be shared
  // - tree sitter languages and queries must be shared

  let mut workers = JoinSet::new();
  for _ in 0..available_concurrency {
    workers.spawn(Worker::new(cache.clone(), config, receiver.clone()).run());
  }
  workers.join_all().await.ok_all()?;

  walker.await?
}
