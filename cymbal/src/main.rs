mod args;
mod cache;
mod ext;
mod symbol;
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

  // TODO:
  // - synchronous walker spawning async worker tasks, joining at the end.
  // - cache must be shared
  // - tree sitter languages and queries must be shared

  Walker::new(cache, &args.path.unwrap()).run().await;

  ().ok()
}
