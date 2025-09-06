use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow)]
pub struct FileInfo {
  // TODO: change to PathBuf
  path: String,

  modified: DateTime<Utc>,
  // TODO: change to enum
  language: String,
  is_fully_parsed: bool,
}
