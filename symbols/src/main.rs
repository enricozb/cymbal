mod args;
mod cache;
mod config;
mod ext;
mod parser;
mod symbol;
mod text;
mod utils;
mod walker;
mod worker;
mod writer;

use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use parking_lot::RwLock;

use crate::{args::Args, walker::Walker, worker::Worker, writer::Writer};

fn main() -> Result<(), anyhow::Error> {
  let args = Args::parse();

  let config = utils::into_static(args.config()?);
  let cache = Arc::new(RwLock::new(args.cache()?));
  let num_threads = args.num_threads();

  let walker = Walker::spawn(config.extensions(), args.num_threads() * 8)?;
  let (writer, writer_handle) = Writer::spawn(args.separator, args.delimiter, args.num_threads() * 8)?;

  let mut worker_handles = Vec::new();
  for _ in 0..num_threads {
    let worker_handle = Worker::new(config, cache.clone(), walker.files.clone(), writer.clone()).spawn();

    worker_handles.push(worker_handle);
  }

  for worker_handle in worker_handles {
    worker_handle.join().expect("join worker").context("failed to join worker")?;
  }

  cache.read().save().context("failed to save cache")?;

  writer.stop()?;
  writer_handle.join().expect("join writer");

  Ok(())
}
