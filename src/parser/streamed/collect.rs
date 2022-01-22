use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::ParseResult;
use crate::parser::Parser;
use crate::stream::Positioned;

use super::StreamedParser;

/// A parser collecting outputs from a [`StreamedParser`].
#[derive(Debug)]
pub struct Collect<P, E> {
    inner: P,
    collection: E,
}

impl<P, E> Collect<P, E>
where
    E: Default,
{
    /// Creating a new instance.
    #[inline]
    pub fn new(parser: P) -> Self {
        Self {
            inner: parser,
            collection: E::default(),
        }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, E, I> Parser<I> for Collect<P, E>
where
    P: StreamedParser<I>,
    E: Default + Extend<P::Output>,
    I: Positioned + ?Sized,
{
    type Output = E;
    type Error = P::Error;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
    ) -> Poll<ParseResult<Self, I>> {
        loop {
            match ready!(self.inner.poll_parse_next(input.as_mut(), cx)?) {
                Some(x) => self.collection.extend(Some(x)),
                None => break Poll::Ready(Ok(mem::take(&mut self.collection))),
            }
        }
    }
}
