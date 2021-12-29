//! Streams used as the input of parsers.
//!
//! The input of parser should implement [`TryStream`] and [`Positioned`] defined here (and
//! sometimes [`Rewind`] will be required), so here we'll provide some implementations
//! on them by wrapping types implementing [`Stream`], [`AsyncRead`], etc.
//!
//! [`Rewind`]: ./trait.Rewind.html
//! [`Positioned`]: ./trait.Positioned.html
//! [`Stream`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
//! [`TryStream`]: https://docs.rs/futures/latest/futures/stream/trait.TryStream.html
//! [`AsyncRead`]: https://docs.rs/futures/latest/futures/io/trait.AsyncRead.html

mod imp;
mod position;
mod rewind;

pub use imp::*;
pub use position::{Positioned, Unpositioned};
pub use rewind::Rewind;
