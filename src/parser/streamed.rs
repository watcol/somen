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
    type Item;

    /// The error type that the stream will returns.
    type Error;

    /// The internal state used in [`poll_parse_next`].
    ///
    /// This state will be initialized by [`Default`] trait and stored in the [`TryStream`].
    ///
    /// [`poll_parse_next`]: Self::poll_parse_next
    /// [`TryStream`]: futures_core::TryStream
    type State: Default;

    /// Takes an input, returns a next output or [`None`].
    fn poll_parse_next(
        &self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<StreamedResult<Self, I>>;
}

/// An extention trait for [`StreamedParser`].
pub trait StreamedParserExt<I: Positioned + ?Sized>: StreamedParser<I> {
    /// Returning a [`TryStream`] by invoking [`poll_parse_next`].
    ///
    /// [`TryStream`]: futures_core::TryStream
    /// [`poll_parse_next`]: self::StreamedParser::poll_parse_next
    #[inline]
    fn parse_streamed<'a, 'b>(
        &'a mut self,
        input: &'b mut I,
    ) -> ParserStream<'a, 'b, Self, I, Self::State>
    where
        I: Unpin,
    {
        ParserStream::new(self, input)
    }

    /// Returns a [`Parser`] by collecting all the outputs.
    ///
    /// [`Parser`]: super::Parser
    #[inline]
    fn collect<E: Default + Extend<Self::Item>>(self) -> Collect<Self, E>
    where
        Self: Sized,
    {
        Collect::new(self)
    }

    /// Wraps the streamed parser into a [`Box`].
    #[cfg(feature = "alloc")]
    #[inline]
    fn boxed<'a>(self) -> BoxStreamedParser<'a, I, Self::Item, Self::Error, Self::State>
    where
        Self: Sized + 'a,
    {
        Box::new(self)
    }

    /// Returns a new streamed parser by wrapping the error into a [`Box`].
    #[cfg(feature = "alloc")]
    #[inline]
    fn box_error(self) -> BoxError<Self>
    where
        Self: Sized,
    {
        BoxError::new(self)
    }
}

impl<P: StreamedParser<I>, I: Positioned + ?Sized> StreamedParserExt<I> for P {}
