mod args;
mod cache;
mod channel;
mod color;
mod config;
mod ext;
mod parser;
mod symbol;
mod symbol_stream;
mod template;
mod utils;
mod walker;
mod worker;

use anyhow::Result;
use clap::Parser;
use tokio::task::JoinSet;

use crate::args::Args;
use crate::ext::{IntoExt, Leak, ResultExt};
use crate::walker::Walker;
use crate::worker::Worker;

#[tokio::main]
async fn main() -> Result<()> {
  let args = Args::parse();
  let cache = args.cache().await?;
  let config = args.config().await?.leak();
  let available_concurrency = args.available_concurrency()?.convert::<usize>();

  let (sender, receiver) = crate::channel::bounded(args.channel_bound);

  // TODO:
  // - synchronous walker spawning async worker tasks, joining at the end.
  // - cache must be shared
  // - tree sitter languages and queries must be shared

  let walker = Walker::new(args.search_path, sender).spawn();

  let mut workers = JoinSet::new();
  for _ in 0..available_concurrency {
    workers.spawn(Worker::new(cache.clone(), config, receiver.clone()).run());
  }

  for res in workers.join_all().await {
    println!("worker result = {res:?}");
  }
  walker.await??;

  ().ok()
}
