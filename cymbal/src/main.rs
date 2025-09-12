mod args;
mod cache;
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

use crate::args::Args;
use crate::ext::ResultExt;
use crate::walker::Walker;

#[tokio::main]
async fn main() -> Result<()> {
  let args = Args::parse();
  let cache = args.cache().await?;
  let config = args.config().await?;

  // TODO:
  // - synchronous walker spawning async worker tasks, joining at the end.
  // - cache must be shared
  // - tree sitter languages and queries must be shared

  Walker::new(&args.search_path, cache, config).run().await;

  ().ok()
}
