use alloc::boxed::Box;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::future::{BoxFuture, FusedFuture, TryFuture};

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
    inner: P,
    _phantom: PhantomData<&'a ()>,
}

impl<P> FutureBoxed<'_, P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(parser: P) -> Self {
        Self {
            inner: parser,
            _phantom: PhantomData,
        }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
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
        Box::pin(self.inner.parse(input))
    }
}

/// A wrapper for parsers to box future objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorBoxed<P> {
    inner: P,
}

impl<P> ErrorBoxed<P> {
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

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct ErrorBoxedFuture<Fut, E, F, L> {
        #[pin]
        inner: Fut,
        _phantom: PhantomData<(E, F, L)>,
    }
}

impl<Fut, E, F, L> FusedFuture for ErrorBoxedFuture<Fut, E, F, L>
where
    Fut: TryFuture<Error = ParseError<E, F, L>> + FusedFuture,
    E: core::fmt::Display + 'static,
{
    #[inline]
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<Fut, E, F, L> Future for ErrorBoxedFuture<Fut, E, F, L>
where
    Fut: TryFuture<Error = ParseError<E, F, L>>,
    E: core::fmt::Display + 'static,
{
    type Output = ParseResult<Fut::Ok, Box<dyn core::fmt::Display + 'static>, F, L>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.try_poll(cx).map_err(|err| match err {
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
    type Future = ErrorBoxedFuture<P::Future, P::Error, I::Error, I::Locator>;

    #[inline]
    fn parse(&'parser self, input: &'input mut I) -> Self::Future {
        ErrorBoxedFuture {
            inner: self.inner.parse(input),
            _phantom: PhantomData,
        }
    }
}
