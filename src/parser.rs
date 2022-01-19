//! Basic parsers and combinators.

mod future;
use future::{ParseFuture, ParsePositionedFuture};

mod boxed;
pub use boxed::BoxParser;

use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, TryStream};

use crate::error::{ParseError, ParseResult, PositionedError, PositionedResult};
use crate::stream::position::Positioned;
use crate::stream::BasicInput;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// A trait for parsers.
pub trait Parser<I: BasicInput + ?Sized> {
    /// The type of the output value.
    type Output;
    /// The type of errors while parsing.
    type Error;

    /// Parse the `input`, get an output.
    fn poll_parse(&self, input: Pin<&mut I>, cx: &mut Context<'_>) -> Poll<ParseResult<Self, I>>;

    /// Parse the `input`, get an output or returning the error information with positions.
    fn poll_parse_positioned(
        &self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
    ) -> Poll<PositionedResult<Self, I>>
    where
        I: Positioned,
    {
        let start = ready!(input
            .as_mut()
            .poll_position(cx)
            .map_err(ParseError::Stream)?);
        let parsed = ready!(self.poll_parse(input.as_mut(), cx));
        let end = ready!(input
            .as_mut()
            .poll_position(cx)
            .map_err(ParseError::Stream)?);

        Poll::Ready(parsed.map_err(|err| match err {
            ParseError::Parser(e) => ParseError::Parser(PositionedError {
                range: start..end,
                error: e,
            }),
            ParseError::Stream(e) => ParseError::Stream(e),
        }))
    }
}

/// A trait for parsers returning multiple outputs with [`Stream`].
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
    fn parser_stream(&self, input: &mut I) -> Self::Stream;
}

pub trait ParserExt<I: BasicInput + ?Sized>: Parser<I> {
    /// An asynchronous version of [`poll_parse`], which returns a [`Future`] object.
    ///
    /// [`poll_parse`]: self::Parser::poll_parse
    /// [`Future`]: core::future::Future
    fn parse<'a>(&self, input: &'a mut I) -> ParseFuture<'_, 'a, Self, I>
    where
        I: Unpin,
    {
        ParseFuture::new(self, input)
    }

    /// An asynchronous version of [`poll_parse_positioned`], which returns a [`Future`] object.
    ///
    /// [`poll_parse_positioned`]: self::Parser::poll_parse_positioned
    /// [`Future`]: core::future::Future
    fn parse_positioned<'a>(&self, input: &'a mut I) -> ParsePositionedFuture<'_, 'a, Self, I>
    where
        I: Positioned + Unpin,
    {
        ParsePositionedFuture::new(self, input)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn boxed(&self) -> BoxParser<'_, I, Self::Output, Self::Error>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl<P: Parser<I>, I: BasicInput + ?Sized> ParserExt<I> for P {}
