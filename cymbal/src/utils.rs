mod one_or_many;

use std::{ffi::OsString, path::PathBuf, sync::LazyLock};

pub use self::one_or_many::*;

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
