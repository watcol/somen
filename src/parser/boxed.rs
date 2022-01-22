use alloc::boxed::Box;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::future::BoxFuture;

use super::Parser;
use crate::error::{ParseError, ParseResult};
use crate::stream::position::Positioned;

/// The boxed parsers.
pub type BoxParser<'a, 'parser, 'input, I, O, E, F> =
    Box<dyn Parser<'parser, 'input, I, Output = O, Error = E, Future = F> + 'a>;

impl<'parser, 'input, I, O, E, F> Parser<'parser, 'input, I>
    for BoxParser<'_, 'parser, 'input, I, O, E, F>
where
    I: Positioned + ?Sized,
    F: Future<Output = Result<O, ParseError<E, I::Error, I::Locator>>>,
{
    type Output = O;
    type Error = E;
    type Future = F;

    #[inline]
    fn parse(&'parser self, input: &'input mut I) -> Self::Future {
        (**self).parse(input)
    }
}

/// A wrapper for parsers to box future objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FutureBoxed<'a, P> {
    parser: P,
    _phantom: PhantomData<&'a ()>,
}

impl<P> FutureBoxed<'_, P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            _phantom: PhantomData,
        }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.parser
    }
}

impl<'a, 'parser, 'input, P, I> Parser<'parser, 'input, I> for FutureBoxed<'a, P>
where
    P: Parser<'parser, 'input, I>,
    P::Future: Send + 'a,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type Error = P::Error;
    #[allow(clippy::type_complexity)]
    type Future = BoxFuture<'a, ParseResult<Self::Output, Self::Error, I::Error, I::Locator>>;

    #[inline]
    fn parse(&'parser self, input: &'input mut I) -> Self::Future {
        Box::pin(self.parser.parse(input))
    }
}

/// A wrapper for parsers to box future objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorBoxed<P> {
    parser: P,
}

impl<P> ErrorBoxed<P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(parser: P) -> Self {
        Self { parser }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.parser
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct ErrorBoxedFuture<Fut, O, E, F, L> {
        #[pin]
        inner: Fut,
        _phantom: PhantomData<(O, E, F, L)>,
    }
}

impl<Fut, O, E, F, L> Future for ErrorBoxedFuture<Fut, O, E, F, L>
where
    Fut: Future<Output = ParseResult<O, E, F, L>>,
    E: core::fmt::Display + 'static,
{
    type Output = ParseResult<O, Box<dyn core::fmt::Display + 'static>, F, L>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx).map_err(|err| match err {
            ParseError::Parser(e, p) => ParseError::Parser(Box::new(e) as _, p),
            ParseError::Stream(e) => ParseError::Stream(e),
        })
    }
}

impl<'parser, 'input, P, I> Parser<'parser, 'input, I> for ErrorBoxed<P>
where
    P: Parser<'parser, 'input, I>,
    P::Error: core::fmt::Display + 'static,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type Error = Box<dyn core::fmt::Display + 'static>;
    #[allow(clippy::type_complexity)]
    type Future = ErrorBoxedFuture<P::Future, P::Output, P::Error, I::Error, I::Locator>;

    #[inline]
    fn parse(&'parser self, input: &'input mut I) -> Self::Future {
        ErrorBoxedFuture {
            inner: self.parser.parse(input),
            _phantom: PhantomData,
        }
    }
}
