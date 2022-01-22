use alloc::boxed::Box;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::Parser;
use crate::error::{ParseError, ParseResult};
use crate::stream::position::Positioned;

/// The boxed parsers.
pub type BoxParser<'a, I, O, E> = Box<dyn Parser<I, Output = O, Error = E> + 'a>;

impl<I, O, E> Parser<I> for BoxParser<'_, I, O, E>
where
    I: Positioned + ?Sized,
{
    type Output = O;
    type Error = E;

    #[inline]
    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
    ) -> Poll<ParseResult<Self, I>> {
        (**self).poll_parse(input, cx)
    }
}

/// A wrapper for parsers to box future objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoxError<P> {
    inner: P,
}

impl<P> BoxError<P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P) -> Self {
        Self { inner }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, I> Parser<I> for BoxError<P>
where
    P: Parser<I>,
    P::Error: core::fmt::Display + 'static,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type Error = Box<dyn core::fmt::Display + 'static>;

    #[inline]
    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
    ) -> Poll<ParseResult<Self, I>> {
        self.inner.poll_parse(input, cx).map_err(|err| match err {
            ParseError::Parser(e, p) => ParseError::Parser(Box::new(e) as _, p),
            ParseError::Stream(e) => ParseError::Stream(e),
        })
    }
}
