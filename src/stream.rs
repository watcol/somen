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

pub mod position;
pub mod record;
pub mod rewind;

pub use position::Positioned;
pub use rewind::Rewind;

/// An alias trait for [`Positioned`]` + `[`Rewind`], full-featured streams.
///
/// [`Positioned`]: self::position::Positioned
/// [`Rewind`]: self::rewind::Rewind
pub trait Input: Positioned + Rewind {}

impl<T: Positioned + Rewind + ?Sized> Input for T {}
