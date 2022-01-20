//! Tools for parsers return multiple outputs.

mod positioned;

pub use positioned::PositionedStream;

use core::pin::Pin;
use futures_core::stream::TryStream;

use crate::error::ParseError;
use crate::stream::position::Positioned;
use crate::stream::BasicInput;

/// A trait for parsers return multiple outputs with [`Stream`].
///
/// [`Stream`]: futures_core::stream::Stream
pub trait StreamedParser<I: BasicInput + ?Sized> {
    /// The type for items of input stream.
    type Output;

    /// The error type that the stream will returns.
    type Error;

    /// The type of returned stream.
    type Stream: TryStream<Ok = Self::Output, Error = ParseError<Self::Error, I::Error>>
        + BorrowMutPin<I>;

    /// Takes an input, returns multiple outputs with [`Stream`].
    ///
    /// [`Stream`]: futures_core::stream::Stream
    fn parser_stream(&self, input: Pin<&mut I>) -> Self::Stream;

    /// A positioned version of [`parser_stream`].
    ///
    /// [`parser_stream`]: Self::parser_stream
    fn parser_stream_positioned(&self, input: Pin<&mut I>) -> PositionedStream<Self::Stream, I>
    where
        Self::Stream: BorrowMutPin<I>,
        I: Positioned,
    {
        PositionedStream::from(self.parser_stream(input))
    }
}

/// Borrowing the input stream which should be owned by parser stream.
///
pub trait BorrowMutPin<I: ?Sized> {
    /// Mutably borrows the pinned input stream.
    fn borrow_mut_pin(self: Pin<&mut Self>) -> Pin<&mut I>;
}
