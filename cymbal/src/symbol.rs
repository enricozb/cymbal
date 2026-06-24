use chrono::{DateTime, Utc};
use enum_assoc::Assoc;
use serde::Deserialize;
use sqlx::Type as SqlxType;

use crate::{
  color::{BLUE, CYAN, GREEN, MAGENTA, YELLOW},
  config::Language,
  utils::Colored,
};

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

impl Symbol {
  pub fn leading_str(&self) -> &str {
    self.leading.as_deref().unwrap_or("")
  }

  pub fn trailing_str(&self) -> &str {
    self.trailing.as_deref().unwrap_or("")
  }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, Assoc, SqlxType)]
#[func(pub const fn color(&self) -> &'static str)]
#[func(pub const fn to_str(&self) -> &'static str)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum Kind {
  #[assoc(to_str = "module ", color = YELLOW)]
  Module,
  #[assoc(to_str = "macro  ", color = YELLOW)]
  Macro,
  #[assoc(to_str = "global ", color = YELLOW)]
  Global,
  #[assoc(to_str = "const  ", color = YELLOW)]
  Constant,
  #[assoc(to_str = "define ", color = YELLOW)]
  Define,

  #[assoc(to_str = "class  ", color = CYAN)]
  Class,
  #[assoc(to_str = "struct ", color = CYAN)]
  Struct,
  #[assoc(to_str = "enum   ", color = CYAN)]
  Enum,
  #[assoc(to_str = "union  ", color = CYAN)]
  Union,

  #[assoc(to_str = "alias  ", color = BLUE)]
  Alias,
  #[assoc(to_str = "inter  ", color = BLUE)]
  Interface,
  #[assoc(to_str = "trait  ", color = BLUE)]
  Trait,
  #[assoc(to_str = "type   ", color = BLUE)]
  Type,

  #[assoc(to_str = "func   ", color = MAGENTA)]
  Function,
  #[assoc(to_str = "method ", color = MAGENTA)]
  Method,
  #[assoc(to_str = "impl   ", color = MAGENTA)]
  Impl,
  #[assoc(to_str = "field  ", color = MAGENTA)]
  Field,

  #[assoc(to_str = "variant", color = GREEN)]
  Variant,

  #[assoc(to_str = "mode   ", color = BLUE)]
  Mode,
  #[assoc(to_str = "hook   ", color = GREEN)]
  Hook,
}

impl Kind {
  pub const fn colored(&self, color: bool) -> Colored {
    Colored {
      string: self.to_str(),
      color: if color { Some(self.color()) } else { None },
    }
  }
}
