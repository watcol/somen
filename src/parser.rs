//! Basic parsers and combinators.

mod future;
use future::ParseFuture;

use core::pin::Pin;
use core::task::{Context, Poll};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::error::ParseResult;
use crate::stream::BasicInput;

#[cfg(feature = "alloc")]
pub type BoxParser<'a, I, O, E> = Box<&'a dyn Parser<I, Output = O, Error = E>>;

/// A trait for parsers.
pub trait Parser<I: BasicInput + ?Sized> {
    /// The type of the output value.
    type Output;
    /// The type of errors while parsing.
    type Error;

    /// Parse the `input`, get an output.
    fn poll_parse(&self, input: Pin<&mut I>, cx: &mut Context<'_>) -> Poll<ParseResult<Self, I>>;
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

#[cfg(feature = "alloc")]
impl<I: BasicInput + ?Sized, O, E> Parser<I> for BoxParser<'_, I, O, E> {
    type Output = O;
    type Error = E;

    #[inline]
    fn poll_parse(&self, input: Pin<&mut I>, cx: &mut Context<'_>) -> Poll<ParseResult<Self, I>> {
        (**self).poll_parse(input, cx)
    }
}
