use std::fmt::Debug;

#[extend::ext(name=Leak)]
pub impl<T> T {
  fn leak(self) -> &'static T {
    Box::leak(Box::new(self))
  }
}

#[extend::ext(name=ResultExt)]
pub impl<T> T {
  fn ok<E>(self) -> Result<T, E> {
    Ok(self)
  }

  fn warn<O, E: Debug>(self) -> Option<O>
  where
    Self: Into<Result<O, E>>,
  {
    match self.into() {
      Ok(t) => t.some(),
      Err(e) => {
        eprintln!("{e:?}");

        None
      }
    }
  }
}

#[extend::ext(name=OptionExt)]
pub impl<T> T {
  fn some(self) -> Option<T> {
    Some(self)
  }

  fn none<E>(self) -> Option<E> {
    None
  }
}
