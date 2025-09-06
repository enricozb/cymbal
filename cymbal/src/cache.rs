use std::path::Path;

use anyhow::{Context, Result};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::ext::ResultExt;

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
