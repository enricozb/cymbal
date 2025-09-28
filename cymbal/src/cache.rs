use std::{
  collections::HashSet,
  path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures::{Stream, StreamExt};
use sqlx::{
  Either, QueryBuilder, Sqlite, SqlitePool,
  sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
};

use crate::{
  ext::{Ignore, IntoExt, PathExt},
  symbol::{FileInfo, Symbol},
  utils::RawPath,
};

#[derive(Clone)]
pub struct Cache {
  pool: SqlitePool,
}

impl Cache {
  const CACHE_FILE_NAME: &'static str = "cymbal-cache.sqlite";

  pub async fn from_dirpath(cache_dir_path: &Path) -> Result<Self> {
    if !cache_dir_path.exists() {
      tokio::fs::create_dir_all(cache_dir_path).await?;
    }

    let cache_filepath = cache_dir_path.join(Self::CACHE_FILE_NAME);
    let options = SqliteConnectOptions::new()
      .filename(cache_filepath)
      .create_if_missing(true)
      .journal_mode(SqliteJournalMode::Wal)
      .synchronous(SqliteSynchronous::Normal);

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

  pub async fn insert_file(&self, file_path: &Path, file_modified: &DateTime<Utc>) -> Result<()> {
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

  async fn insert_symbols_impl(&self, file_path_bytes: &[u8], symbols: &[Symbol]) -> Result<()> {
    let mut query = QueryBuilder::new("INSERT INTO symbol (file_path, kind, language, line, column, content, leading, trailing)");
    query.push_values(symbols, |mut query, symbol| {
      query.push_bind(file_path_bytes);
      query.push_bind(symbol.kind);
      query.push_bind(symbol.language);
      query.push_bind(symbol.line);
      query.push_bind(symbol.column);
      query.push_bind(&symbol.content);
      query.push_bind(&symbol.leading);
      query.push_bind(&symbol.trailing);
    });

    query.build().execute(&self.pool).await.context("failed to insert symbols")?;

    ().ok()
  }

  pub async fn insert_symbols(&self, file_path: &Path, symbols: &[Symbol]) -> Result<()> {
    let file_path_bytes = file_path.as_bytes();

    sqlx::query("DELETE FROM symbol WHERE file_path = $1")
      .bind(file_path_bytes)
      .execute(&self.pool)
      .await
      .context("failed to delete stale symbols")?;

    for symbols_chunk in symbols.chunks(100) {
      self.insert_symbols_impl(file_path_bytes, symbols_chunk).await?;
    }

    ().ok()
  }

  pub async fn set_file_is_fully_parsed(&self, file_path: &Path) -> Result<()> {
    sqlx::query("UPDATE file SET is_fully_parsed = TRUE WHERE path = $1")
      .bind(file_path.as_bytes())
      .execute(&self.pool)
      .await
      .map(Ignore::ignore)
      .context("failed to set is_fully_parsed for file info")
  }

  pub fn get_symbols<'a, 'path>(&'a self, file_path: &'path Path) -> impl Stream<Item = Result<Symbol, sqlx::Error>> + 'path
  where
    'a: 'path,
  {
    sqlx::query_as("SELECT * FROM symbol WHERE symbol.file_path = $1")
      .bind(file_path.as_bytes())
      .fetch_many(&self.pool)
      .filter_map(async |row| row.map(Either::right).transpose())
  }

  pub async fn delete_stale_file_paths(&self, file_paths: &HashSet<PathBuf>) -> Result<()> {
    let cached_file_paths = self.get_file_paths();
    futures::pin_mut!(cached_file_paths);

    while let Some(file_path) = cached_file_paths.next().await {
      if !file_paths.contains(&file_path) {
        self.delete_file(&file_path).await?;
      }
    }

    ().ok()
  }

  async fn delete_file(&self, file_path: &Path) -> Result<()> {
    let file_path_bytes = file_path.as_bytes();

    sqlx::query("DELETE FROM file WHERE path = $1")
      .bind(file_path_bytes)
      .execute(&self.pool)
      .await
      .context("failed to file")?;

    ().ok()
  }

  fn get_file_paths(&self) -> impl Stream<Item = PathBuf> {
    sqlx::query_as::<Sqlite, RawPath>("SELECT path FROM file")
      .fetch_many(&self.pool)
      .filter_map(async |row| row.ok()?.right()?.convert::<PathBuf>().some())
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
