mod one_or_many;

use std::{ffi::OsString, fmt::Display, path::PathBuf, sync::LazyLock};

pub use self::one_or_many::*;
use crate::color::RESET;

pub type Lazy<T> = LazyLock<T, Box<dyn FnOnce() -> T + Send>>;

#[derive(Clone, Debug, sqlx::Type, sqlx::FromRow)]
#[sqlx(transparent)]
pub struct RawPath(Vec<u8>);

impl From<PathBuf> for RawPath {
  fn from(file_path: PathBuf) -> Self {
    Self(file_path.into_os_string().into_encoded_bytes())
  }
}

impl From<RawPath> for PathBuf {
  fn from(RawPath(bytes): RawPath) -> Self {
    let os_string = unsafe { OsString::from_encoded_bytes_unchecked(bytes) };

    os_string.into()
  }
}

pub struct Colored {
  pub string: &'static str,
  pub color: Option<&'static str>,
}

impl Display for Colored {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let Some(color) = self.color {
      write!(f, "{}{}{RESET}", color, self.string)
    } else {
      write!(f, "{}", self.string)
    }
  }
}
