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
/// [`collect`]: super::StreamedParserExt::collect
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

/// A parser for method [`count`].
///
/// [`count`]: super::StreamedParserExt::count
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Count<P> {
    inner: P,
}

impl<P> Count<P> {
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CountState<C> {
    inner: C,
    count: usize,
}

impl<P, I> Parser<I> for Count<P>
where
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Output = usize;
    type State = CountState<P::State>;

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
                Some(_) => state.count += 1,
                None => break Poll::Ready(Ok(mem::take(&mut state.count))),
            }
        }
    }
}

/// A parser for method [`discard`].
///
/// [`discard`]: super::StreamedParserExt::discard
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Discard<P> {
    inner: P,
}

impl<P> Discard<P> {
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

impl<P, I> Parser<I> for Discard<P>
where
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Output = ();
    type State = P::State;

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
                state,
                tracker
            )?) {
                Some(_) => continue,
                None => break Poll::Ready(Ok(())),
            }
        }
    }
}
