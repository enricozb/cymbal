#[extend::ext(name=ResultExt)]
pub impl<T> T {
  fn ok<E>(self) -> Result<T, E> {
    Ok(self)
  }
}
