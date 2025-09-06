mod args;
mod cache;
mod ext;

use anyhow::Result;
use clap::Parser;

use crate::args::Args;
use crate::cache::Cache;
use crate::ext::ResultExt;

#[tokio::main]
async fn main() -> Result<()> {
  let args = Args::parse();
  let cache = args.cache().await?;

  // TODO:
  // - synchronous walker spawning async worker tasks, joining at the end.
  // - cache must be shared
  // - tree sitter languages and queries must be shared

  ().ok()
}
