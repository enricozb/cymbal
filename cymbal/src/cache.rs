use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures::{Stream, StreamExt};
use sqlx::{Either, SqlitePool, sqlite::SqliteConnectOptions};

use crate::{
  ext::{Ignore, IntoExt, PathBufExt, PathExt},
  symbol::{FileInfo, Symbol},
};

#[derive(Clone)]
pub struct Cache {
  pool: SqlitePool,
}

impl Cache {
  const CACHE_FILE_NAME: &'static str = "cymbal-cache.sqlite";

  pub async fn from_dirpath(cache_dirpath: &Path) -> Result<Self> {
    let cache_filepath = cache_dirpath.join(Self::CACHE_FILE_NAME);
    let options = SqliteConnectOptions::new().filename(cache_filepath).create_if_missing(true);

    Self::from_options(options).await
  }

  pub async fn is_file_cached(&self, file_path: &Path, file_modified: &DateTime<Utc>) -> Result<bool> {
    // TODO(enricozb): try writing an EXISTS query to check performance
    let Some(cache_file_info) = self.get_file_info(file_path).await? else { return false.ok() };
    let is_cached = &cache_file_info.modified == file_modified && cache_file_info.is_fully_parsed;

    is_cached.ok()
  }

  async fn get_file_info(&self, file_path: &Path) -> Result<Option<FileInfo>> {
    sqlx::query_as("SELECT modified, is_fully_parsed FROM file WHERE file.path = $1")
      .bind(file_path.as_bytes())
      .fetch_optional(&self.pool)
      .await
      .context("failed to get file info")
  }

  pub async fn insert_file_info(&self, file_path: &Path, file_modified: &DateTime<Utc>) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO file (path, modified)
          VALUES ($1, $2)
        ON CONFLICT DO UPDATE SET
          modified = excluded.modified,
          is_fully_parsed = FALSE
      ",
    )
    .bind(file_path.as_bytes())
    .bind(file_modified)
    .execute(&self.pool)
    .await
    .map(Ignore::ignore)
    .context("failed to insert file info")
  }

  pub async fn insert_symbol(&self, file_path: &Path, symbol: &Symbol) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO symbol (file_path, kind, language, line, column, content, leading, trailing)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT DO NOTHING
      ",
    )
    .bind(file_path.as_bytes())
    .bind(symbol.kind)
    .bind(symbol.language)
    .bind(symbol.line)
    .bind(symbol.column)
    .bind(&symbol.content)
    .bind(&symbol.leading)
    .bind(&symbol.trailing)
    .execute(&self.pool)
    .await
    .map(Ignore::ignore)
    .context("failed to insert file info")
  }

  pub async fn set_is_fully_parsed(&self, file_path: &Path) -> Result<()> {
    sqlx::query("UPDATE file SET is_fully_parsed = TRUE WHERE path = $1")
      .bind(file_path.as_bytes())
      .execute(&self.pool)
      .await
      .map(Ignore::ignore)
      .context("failed to set is_fully_parsed for file info")
  }

  pub fn symbols<'a, 'path>(&'a self, file_path: &'path Path) -> impl Stream<Item = Result<Symbol, sqlx::Error>> + 'path
  where
    'a: 'path,
  {
    sqlx::query_as("SELECT * FROM symbol WHERE symbol.file_path = $1")
      .bind(file_path.as_bytes())
      .fetch_many(&self.pool)
      .filter_map(async |row| row.map(Either::right).transpose())
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
      .context("failed to migrate")?;

    ().ok()
  }
}
