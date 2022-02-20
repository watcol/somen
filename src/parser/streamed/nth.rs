use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

use super::StreamedParser;

/// A parser for method [`nth`].
///
/// [`nth`]: super::StreamedParserExt::nth
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nth<P> {
    inner: P,
    n: usize,
}

impl<P> Nth<P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, n: usize) -> Self {
        Self { inner, n }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NthState<C, T> {
    inner: C,
    count: usize,
    res: Option<T>,
}

impl<C: Default, T> Default for NthState<C, T> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: C::default(),
            count: 0,
            res: None,
        }
    }
}

impl<P, I> Parser<I> for Nth<P>
where
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Output = Option<P::Item>;
    type State = NthState<P::State, P::Item>;

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
                Some(val) => {
                    if state.count == self.n {
                        state.res = Some(val);
                    }
                    state.count += 1;
                }
                None => break Poll::Ready(Ok(mem::take(&mut state.res))),
            }
        }
    }
}

/// A parser for method [`last`].
///
/// [`last`]: super::StreamedParserExt::last
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Last<P> {
    inner: P,
}

impl<P> Last<P> {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LastState<C, T> {
    inner: C,
    last: Option<T>,
}

impl<C: Default, T> Default for LastState<C, T> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: C::default(),
            last: None,
        }
    }
}

impl<P, I> Parser<I> for Last<P>
where
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Output = Option<P::Item>;
    type State = LastState<P::State, P::Item>;

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
                Some(val) => {
                    state.last = Some(val);
                }
                None => break Poll::Ready(Ok(mem::take(&mut state.last))),
            }
        }
    }
}
