use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::Type as SqlxType;

use crate::color;
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

impl Symbol {
  pub fn leading_str(&self) -> &str {
    self.leading.as_deref().unwrap_or("")
  }

  pub fn trailing_str(&self) -> &str {
    self.trailing.as_deref().unwrap_or("")
  }
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

impl Kind {
  #[rustfmt::skip]
  pub fn colored_abbreviation(self) -> &'static str {
    // TODO(enricozb): have some macro generate this to automatically pad all
    // symbols.
    // Note: these strings must all have the same printable length.
    match self {
      Self::Module    => color!("(mod)   ", yellow),
      Self::Macro     => color!("(macro) ", yellow),
      Self::Global    => color!("(global)", yellow),
      Self::Constant  => color!("(const) ", yellow),
      Self::Define    => color!("(define)", yellow),

      Self::Class     => color!("(class) ", cyan),
      Self::Struct    => color!("(struct)", cyan),
      Self::Enum      => color!("(enum)  ", cyan),
      Self::Union     => color!("(union) ", cyan),

      Self::Alias     => color!("(alias) ", blue),
      Self::Interface => color!("(inter) ", blue),
      Self::Trait     => color!("(trait) ", blue),
      Self::Type      => color!("(type)  ", blue),

      Self::Function  => color!("(func)  ", magenta),
      Self::Method    => color!("(method)", magenta),
      Self::Impl      => color!("(impl)  ", magenta),
    }
  }
}
