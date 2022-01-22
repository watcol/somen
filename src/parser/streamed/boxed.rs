use alloc::boxed::Box;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::StreamedParser;
use crate::error::{ParseError, StreamedResult};
use crate::stream::position::Positioned;

/// The boxed streamed parsers.
pub type BoxStreamedParser<'a, I, T, E> = Box<dyn StreamedParser<I, Item = T, Error = E> + 'a>;

impl<I, T, E> StreamedParser<I> for BoxStreamedParser<'_, I, T, E>
where
    I: Positioned + ?Sized,
{
    type Item = T;
    type Error = E;

    #[inline]
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
    ) -> Poll<StreamedResult<Self, I>> {
        (**self).poll_parse_next(input, cx)
    }
}

/// A wrapper for streamed parsers to box future objects.
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

impl<P, I> StreamedParser<I> for BoxError<P>
where
    P: StreamedParser<I>,
    P::Error: core::fmt::Display + 'static,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type Error = Box<dyn core::fmt::Display + 'static>;

    #[inline]
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
    ) -> Poll<StreamedResult<Self, I>> {
        self.inner
            .poll_parse_next(input, cx)
            .map_err(|err| match err {
                ParseError::Parser(e, p) => ParseError::Parser(Box::new(e) as _, p),
                ParseError::Stream(e) => ParseError::Stream(e),
            })
    }
}
