//! Tools for parsers return multiple outputs.

#[cfg(feature = "alloc")]
mod boxed;
mod collect;
mod stream;

use core::pin::Pin;
use core::task::{Context, Poll};

#[cfg(feature = "alloc")]
pub use boxed::{BoxError, BoxStreamedParser};
pub use collect::Collect;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::error::StreamedResult;
use crate::stream::position::Positioned;
use stream::ParserStream;

/// A trait for parsers return multiple outputs with [`TryStream`].
///
/// [`TryStream`]: futures_core::TryStream
pub trait StreamedParser<I: Positioned + ?Sized> {
    /// The type for items of input stream.
    type Output;

    /// The error type that the stream will returns.
    type Error;

    /// Takes an input, returns a next output or [`None`].
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
    ) -> Poll<StreamedResult<Self, I>>;
}

/// An extention trait for [`StreamedParser`].
pub trait StreamedParserExt<I: Positioned + ?Sized>: StreamedParser<I> {
    /// Returning a [`TryStream`] by invoking [`poll_parse_next`].
    ///
    /// [`TryStream`]: futures_core::TryStream
    /// [`poll_parse_next`]: Self::poll_parse_next
    fn parse_streamed<'a, 'b>(&'a mut self, input: &'b mut I) -> ParserStream<'a, 'b, Self, I> {
        ParserStream::new(self, input)
    }

    /// Returns a [`Parser`] by collecting all the outputs.
    ///
    /// [`Parser`]: super::Parser
    fn collect<E: Default + Extend<Self::Output>>(self) -> Collect<Self, E>
    where
        Self: Sized,
    {
        Collect::new(self)
    }

    /// Wraps the streamed parser into a [`Box`].
    #[cfg(feature = "alloc")]
    fn boxed<'a>(self) -> BoxStreamedParser<'a, I, Self::Output, Self::Error>
    where
        Self: Sized + 'a,
    {
        Box::new(self)
    }

    /// Returns a new streamed parser by wrapping the error into a [`Box`].
    #[cfg(feature = "alloc")]
    fn box_error(self) -> BoxError<Self>
    where
        Self: Sized,
    {
        BoxError::new(self)
    }
}
