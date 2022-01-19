//! Tools for parsers return multiple outputs.

use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;
use futures_core::stream::{Stream, TryStream};
use pin_project_lite::pin_project;

use crate::error::{ParseError, PositionedError};
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
        + BorrowInput<I>;

    /// Takes an input, returns multiple outputs with [`Stream`].
    ///
    /// [`Stream`]: futures_core::stream::Stream
    fn parser_stream(&self, input: &mut I) -> Self::Stream;

    /// A positioned version of [`parser_stream`].
    ///
    /// [`parser_stream`]: Self::parser_stream
    fn parser_stream_positioned(&self, input: &mut I) -> PositionedStream<Self::Stream, I>
    where
        Self::Stream: BorrowInput<I>,
        I: Positioned,
    {
        PositionedStream::from(self.parser_stream(input))
    }
}

/// Borrowing the input stream which should be owned by parser stream.
///
pub trait BorrowInput<I: ?Sized> {
    /// Mutably borrows the pinned input stream.
    fn borrow_mut(self: Pin<&mut Self>) -> Pin<&mut I>;
}

pin_project! {
    /// A wrapping stream type for [`parser_stream_positioned`].
    ///
    /// [`parser_stream_positioned`]: self::StreamedParser::parser_stream_positioned
    #[derive(Debug)]
    pub struct PositionedStream<S, I: ?Sized> {
        #[pin]
        inner : S,
        _phantom: PhantomData<I>,
    }
}

impl<S, I: ?Sized> From<S> for PositionedStream<S, I> {
    #[inline]
    fn from(inner: S) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<S, I: ?Sized> PositionedStream<S, I> {
    /// Extracting the inner stream.
    #[inline]
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S, I, E, F> Stream for PositionedStream<S, I>
where
    S: TryStream<Error = ParseError<E, F>> + BorrowInput<I>,
    I: BasicInput<Error = F> + Positioned + ?Sized,
{
    #[allow(clippy::type_complexity)]
    type Item = Result<S::Ok, ParseError<PositionedError<E, I::Position>, F>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let start = match ready!(this.inner.as_mut().borrow_mut().poll_position(cx)) {
            Ok(s) => s,
            Err(e) => return Poll::Ready(Some(Err(ParseError::Stream(e)))),
        };
        let parsed = ready!(this.inner.as_mut().try_poll_next(cx));
        let end = match ready!(this.inner.as_mut().borrow_mut().poll_position(cx)) {
            Ok(s) => s,
            Err(e) => return Poll::Ready(Some(Err(ParseError::Stream(e)))),
        };
        match parsed {
            Some(Ok(i)) => Poll::Ready(Some(Ok(i))),
            Some(Err(err)) => match err {
                ParseError::Parser(e) => {
                    Poll::Ready(Some(Err(ParseError::Parser(PositionedError {
                        range: start..end,
                        error: e,
                    }))))
                }
                ParseError::Stream(e) => Poll::Ready(Some(Err(ParseError::Stream(e)))),
            },
            None => Poll::Ready(None),
        }
    }
}
