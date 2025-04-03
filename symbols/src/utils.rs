use serde::Deserialize;

pub fn into_static<T>(t: T) -> &'static T {
  Box::leak(Box::new(t))
}

#[derive(Deserialize)]
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
