use core::marker::PhantomData;
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
    _phantom: PhantomData<E>,
}

impl<P, E> Collect<P, E> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Debug, Default)]
pub struct CollectState<C, E> {
    inner: C,
    collection: E,
}

impl<P, E, I> Parser<I> for Collect<P, E>
where
    P: StreamedParser<I>,
    E: Default + Extend<P::Item>,
    I: Positioned + ?Sized,
{
    type Output = E;
    type Error = P::Error;
    type State = CollectState<P::State, E>;

    fn poll_parse(
        &self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        loop {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
            {
                Some(x) => state.collection.extend(Some(x)),
                None => break Poll::Ready(Ok(mem::take(&mut state.collection))),
            }
        }
    }
}
