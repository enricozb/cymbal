mod one_or_many;

use std::sync::LazyLock;

pub use self::one_or_many::*;

pub type Lazy<T> = LazyLock<T, Box<dyn FnOnce() -> T + Send>>;
