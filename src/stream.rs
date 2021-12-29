//! Streams used as the input of parsers.
//!
//! The input of parser should implement `futures::stream::TryStream` and `Positioned` defined
//! here (and sometimes `Rewind` will be required), so here we'll provide some implementations
//! on them by wrapping types implementing `futures::stream::Stream`, `futures::io::AsyncRead`,
//! etc.

mod position;
mod rewind;

pub use position::Positioned;
pub use rewind::Rewind;
