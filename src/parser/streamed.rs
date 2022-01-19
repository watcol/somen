//! Tools for parsers return multiple outputs.

use crate::stream::BasicInput;
use futures_core::stream::TryStream;

/// A trait for parsers return multiple outputs with [`Stream`].
///
/// [`Stream`]: futures_core::stream::Stream
pub trait StreamedParser<I: BasicInput + ?Sized> {
    /// The type for items of input stream.
    type Output;

    /// The error type that the stream will returns.
    type Error;

    /// The type of returned stream.
    type Stream: TryStream<Ok = Self::Output, Error = Self::Error>;

    /// Takes an input, returns multiple outputs with [`Stream`].
    ///
    /// [`Stream`]: futures_core::stream::Stream
    fn parser_stream(&self, input: &mut I) -> Self::Stream;
}
