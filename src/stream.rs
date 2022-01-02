//! Streams used as the input of parsers.
//!
//! The input of parser should implement [`TryStream`] and [`Positioned`] defined here (and
//! sometimes [`Rewind`] will be required), so here we'll provide some implementations
//! on them by wrapping types implementing [`Stream`], [`AsyncRead`], etc.
//!
//! [`Rewind`]: crate::stream::rewind::Rewind
//! [`Positioned`]: crate::stream::position::Positioned
//! [`Stream`]: futures_core::stream::Stream
//! [`TryStream`]: futures_core::stream::TryStream
//! [`AsyncRead`]: futures_io::AsyncRead

mod builder;
mod imp;
pub use builder::*;
pub use imp::*;

pub mod convert;
pub mod position;
pub mod record;
pub mod rewind;

use position::Positioned;
use rewind::Rewind;
