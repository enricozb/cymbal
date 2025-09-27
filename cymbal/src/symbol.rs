use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::Type as SqlxType;

use crate::config::Language;

#[derive(sqlx::FromRow)]
pub struct FileInfo {
  pub modified: DateTime<Utc>,
  pub is_fully_parsed: bool,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Symbol {
  pub kind: Kind,
  pub language: Language,
  pub line: i64,
  pub column: i64,
  pub content: String,
  pub leading: Option<String>,
  pub trailing: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, SqlxType)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum Kind {
  Module,
  Macro,
  Global,
  Constant,
  Define,

  Class,
  Struct,
  Enum,
  Union,

  Alias,
  Interface,
  Trait,
  Type,

  Function,
  Method,
  Impl,
}
