use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use futures::{Stream, StreamExt};
use sqlx::{Either, SqlitePool, sqlite::SqliteConnectOptions};

use crate::{
  ext::{Ignore, PathExt, ResultExt},
  symbol::{FileInfo, Symbol},
};

#[derive(Clone)]
pub struct Cache {
  pool: SqlitePool,
}

impl Cache {
  const CACHE_FILE_NAME: &'static str = "cymbal-cache.sqlite";

  pub async fn new() -> Result<Self> {
    let options = SqliteConnectOptions::new().filename("/tmp/cymbal").create_if_missing(true);

    Self::from_options(options).await
  }

  pub async fn from_dirpath(cache_dirpath: &Path) -> Result<Self> {
    let cache_filepath = cache_dirpath.join(Self::CACHE_FILE_NAME);
    let options = SqliteConnectOptions::new().filename(cache_filepath).create_if_missing(true);

    Self::from_options(options).await
  }

  pub async fn get_file_info(&self, file_path: &Path) -> Result<Option<FileInfo>> {
    sqlx::query_as("SELECT * FROM file WHERE file.path = $1")
      .bind(file_path.to_string_lossy().into_owned())
      .fetch_optional(&self.pool)
      .await
      .context("failed to get file info")
  }

  pub async fn insert_file_info(&self, file_info: &FileInfo) -> Result<()> {
    sqlx::query(
      "
        INSERT INTO file (path, modified, is_fully_parsed)
          VALUES ($1, $2, $3)
        ON CONFLICT DO UPDATE SET
          modified = excluded.modified,
          is_fully_parsed = excluded.is_fully_parsed
      ",
    )
    .bind(&file_info.path)
    .bind(file_info.modified)
    .bind(file_info.is_fully_parsed)
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
    .bind(file_path.to_string_lossy())
    .bind(symbol.kind)
    .bind(symbol.language)
    .bind(symbol.line as i64)
    .bind(symbol.column as i64)
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
      .bind(file_path.into_owned_string_lossy())
      .execute(&self.pool)
      .await
      .map(Ignore::ignore)
      .context("failed to set is_fully_parsed for file info")
  }

  pub fn symbols(&self, file_path: &Path) -> impl Stream<Item = Result<Symbol, sqlx::Error>> {
    sqlx::query_as("SELECT * FROM symbol WHERE symbol.file_path = $1")
      .bind(file_path.into_owned_string_lossy())
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

    println!("initialized...");

    ().ok()
  }
}
