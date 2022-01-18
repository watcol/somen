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

use futures_core::stream::TryStream;
use position::Positioned;
use rewind::Rewind;

/// An alias trait for [`TryStream`]` + `[`Positioned`]` + `[`Rewind`], the most featured parser.
///
/// [`TryStream`]: futures_core::stream::TryStream
/// [`Positioned`]: self::position::Positioned
/// [`Rewind`]: self::rewind::Rewind
pub trait Input: TryStream + Positioned + Rewind {
    type Token;
}

impl<T: TryStream + Positioned + Rewind + ?Sized> Input for T {
    type Token = T::Ok;
}

/// An alias trait for [`TryStream`]` + `[`Positioned`].
pub trait PositionedInput: TryStream + Positioned {
    type Token;
}

impl<T: TryStream + Positioned + ?Sized> PositionedInput for T {
    type Token = T::Ok;
}

/// An alias trait for [`TryStream`]` + `[`Rewind`].
pub trait RewindableInput: TryStream + Rewind {
    type Token;
}

impl<T: TryStream + Rewind + ?Sized> RewindableInput for T {
    type Token = T::Ok;
}

/// An alias trait for [`TryStream`].
pub trait BasicInput: TryStream {
    type Token;
}

impl<T: TryStream + ?Sized> BasicInput for T {
    type Token = T::Ok;
}
