use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
  One(T),
  Many(Vec<T>),
}

impl<T> From<OneOrMany<T>> for Vec<T> {
  fn from(from: OneOrMany<T>) -> Self {
    match from {
      OneOrMany::One(val) => vec![val],
      OneOrMany::Many(vec) => vec,
    }
  }
}

impl<T> IntoIterator for OneOrMany<T> {
  type Item = T;
  type IntoIter = std::vec::IntoIter<T>;

  fn into_iter(self) -> Self::IntoIter {
    match self {
      OneOrMany::One(val) => vec![val].into_iter(),
      OneOrMany::Many(vec) => vec.into_iter(),
    }
  }
}
