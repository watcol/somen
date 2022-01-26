use alloc::boxed::Box;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::Parser;
use crate::error::{ParseError, ParseResult};
use crate::stream::position::Positioned;

/// The boxed parsers.
pub type BoxParser<'a, I, O, E, C> = Box<dyn Parser<I, Output = O, Error = E, State = C> + 'a>;

impl<I, O, E, C> Parser<I> for BoxParser<'_, I, O, E, C>
where
    I: Positioned + ?Sized,
    C: Default,
{
    type Output = O;
    type Error = E;
    type State = C;

    #[inline]
    fn poll_parse(
        &self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        (**self).poll_parse(input, cx, state)
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
    type State = P::State;

    #[inline]
    fn poll_parse(
        &self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        self.inner
            .poll_parse(input, cx, state)
            .map_err(|err| match err {
                ParseError::Parser(e, p) => ParseError::Parser(Box::new(e) as _, p),
                ParseError::Stream(e) => ParseError::Stream(e),
            })
    }
}

/// A wrapper for parsers to box future objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoxState<P> {
    inner: P,
}

impl<P> BoxState<P> {
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

impl<P, I> Parser<I> for BoxState<P>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type Error = P::Error;
    type State = Box<P::State>;

    #[inline]
    fn poll_parse(
        &self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        self.inner.poll_parse(input, cx, state)
    }
}
