//! Streams used as the input of parsers.
//!
//! The input of parser should implement [`TryStream`] and [`Positioned`] defined here (and
//! sometimes [`Rewind`] will be required), so here we'll provide some implementations
//! on them by wrapping types implementing [`Stream`], [`AsyncRead`], etc.
//!
//! [`Rewind`]: crate::stream::Rewind
//! [`Positioned`]: crate::stream::Positioned
//! [`Stream`]: futures_core::stream::Stream
//! [`TryStream`]: futures_core::stream::TryStream
//! [`AsyncRead`]: futures_io::AsyncRead

mod imp;
mod position;
mod rewind;

pub use imp::*;
pub use position::{Positioned, Unpositioned};
pub use rewind::Rewind;
