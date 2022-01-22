//! Tools for parsers return multiple outputs.

use core::pin::Pin;
use futures_core::stream::TryStream;

use crate::error::ParseError;
use crate::stream::position::Positioned;

/// A trait for parsers return multiple outputs with [`TryStream`].
pub trait StreamedParser<I: Positioned + ?Sized> {
    /// The type for items of input stream.
    type Output;

    /// The error type that the stream will returns.
    type Error;

    /// The type of returned stream.
    type Stream: TryStream<Ok = Self::Output, Error = ParseError<Self::Error, I::Error, I::Locator>>;

    /// Takes an input, returns multiple outputs with [`TryStream`].
    fn parse_streamed(&self, input: Pin<&mut I>) -> Self::Stream;
}
