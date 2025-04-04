use serde::{Deserialize, Serialize};

use crate::text::Span;

#[derive(Default, Serialize, Deserialize)]
pub struct Symbol<P = (), T = String> {
  pub path: P,
  pub span: Span,
  pub lead: T,
  pub text: T,
  pub tail: T,
  pub kind: Kind,
}

impl<P, T> Symbol<P, T> {
  pub fn forget_path<T2>(self) -> Symbol<(), T2>
  where
    T: Into<T2>,
  {
    Symbol {
      path: (),
      span: self.span,
      lead: self.lead.into(),
      text: self.text.into(),
      tail: self.tail.into(),
      kind: self.kind,
    }
  }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
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

  Unknown,
}

impl Default for Kind {
  fn default() -> Self {
    Self::Unknown
  }
}

impl Kind {
  #[rustfmt::skip]
  pub fn colored_abbreviation(self) -> &'static str {
    // TODO(enricozb): consider using a const function or crabtime to implement
    // this more readably.
    // these strings must all have the same printable length
    match self {
      Self::Module    => "\x1b[33m(mod)   \x1b[0m",
      Self::Macro     => "\x1b[33m(macro) \x1b[0m",
      Self::Global    => "\x1b[33m(global)\x1b[0m",
      Self::Constant  => "\x1b[33m(const) \x1b[0m",
      Self::Define    => "\x1b[33m(define)\x1b[0m",

      Self::Class     => "\x1b[36m(class) \x1b[0m",
      Self::Struct    => "\x1b[36m(struct)\x1b[0m",
      Self::Enum      => "\x1b[36m(enum)  \x1b[0m",
      Self::Union     => "\x1b[36m(union) \x1b[0m",

      Self::Alias     => "\x1b[34m(alias) \x1b[0m",
      Self::Interface => "\x1b[34m(inter) \x1b[0m",
      Self::Trait     => "\x1b[34m(trait) \x1b[0m",
      Self::Type      => "\x1b[34m(type)  \x1b[0m",

      Self::Function  => "\x1b[35m(func)  \x1b[0m",
      Self::Method    => "\x1b[35m(method)\x1b[0m",
      Self::Impl      => "\x1b[35m(impl)  \x1b[0m",

      Self::Unknown   => "\x1b[31m(??????)\x1b[0m",
    }
  }
}
