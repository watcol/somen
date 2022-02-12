use core::marker::PhantomData;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

use super::StreamedParser;

/// A parser for method [`collect`].
///
/// [`collect`]: super::StreamedParser::collect
#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
    type State = CollectState<P::State, E>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        loop {
            match ready!(self.inner.poll_parse_next(
                input.as_mut(),
                cx,
                &mut state.inner,
                tracker
            )?) {
                Some(x) => state.collection.extend(Some(x)),
                None => break Poll::Ready(Ok(mem::take(&mut state.collection))),
            }
        }
    }
}
