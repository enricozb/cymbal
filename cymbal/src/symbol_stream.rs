use anyhow::Result;
use futures::{Stream, StreamExt, future::Either};

use crate::{ext::ResultExt, symbol::Symbol};

pub trait ParserSymbolStream: Stream<Item = Symbol> {}
impl<T: Stream<Item = Symbol>> ParserSymbolStream for T {}

pub trait CacheSymbolStream: Stream<Item = Result<Symbol, sqlx::Error>> {}
impl<T: Stream<Item = Result<Symbol, sqlx::Error>>> CacheSymbolStream for T {}

pub enum SymbolStream<P, C> {
  FromParser(P),
  FromCache(C),
}

impl<P, C> SymbolStream<P, C>
where
  P: ParserSymbolStream,
  C: CacheSymbolStream,
{
  pub fn is_from_parser(&self) -> bool {
    matches!(self, Self::FromParser(_))
  }

  pub fn into_stream(self) -> impl Stream<Item = Result<Symbol>> {
    match self {
      Self::FromParser(p) => Either::Left(p.map(ResultExt::ok)),
      Self::FromCache(c) => Either::Right(c.map(ResultExt::into_anyhow)),
    }
  }
}
