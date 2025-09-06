use std::path::Path;

use anyhow::{Context, Result};
use futures::Stream;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::{ext::ResultExt, symbol::FileInfo};

type SqlxResult<T> = Result<T, sqlx::Error>;

#[derive(Clone)]
pub struct Cache {
  pool: SqlitePool,
}

impl Cache {
  const CACHE_FILENAME: &'static str = "cymbal-cache.sqlite";

  pub async fn new() -> Result<Self> {
    let options = SqliteConnectOptions::new().in_memory(true);

    Self::from_options(options).await
  }

  pub async fn from_dirpath(cache_dirpath: &Path) -> Result<Self> {
    let cache_filepath = cache_dirpath.join(Self::CACHE_FILENAME);
    let options = SqliteConnectOptions::new()
      .filename(cache_filepath)
      .create_if_missing(true);

    Self::from_options(options).await
  }

  pub fn symbols(&self, filepath: &Path) -> impl Stream<Item = SqlxResult<FileInfo>> {
    sqlx::query_as("SELECT * FROM file").fetch(&self.pool)
  }

  async fn from_options(options: SqliteConnectOptions) -> Result<Self> {
    let pool = SqlitePool::connect_with(options)
      .await
      .context("failed to create sqlite connection pool")?;
    let cache = Self { pool };

    cache.initialize().await?;

    cache.ok()
  }

  async fn initialize(&self) -> Result<()> {
    sqlx::migrate!("src/cache/migrations")
      .run(&self.pool)
      .await
      .context("failed to migrate")
  }
}
