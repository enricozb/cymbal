use serde::{Deserialize, Serialize};

use crate::{color, text::Span};

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

      Self::Unknown   => color!("(??????)", red),
    }
  }
}
