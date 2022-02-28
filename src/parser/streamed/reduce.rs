use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Expects, ParseError, PolledResult, Tracker};
use crate::parser::utils::SpanState;
use crate::parser::Parser;
use crate::stream::Positioned;

use super::StreamedParser;

/// A parser for method [`reduce`].
///
/// [`reduce`]: super::StreamedParserExt::reduce
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reduce<P, F> {
    inner: P,
    f: F,
}

impl<P, F> Reduce<P, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReduceState<C, T> {
    inner: C,
    acc: Option<T>,
    committed: bool,
}

impl<C: Default, T> Default for ReduceState<C, T> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: C::default(),
            acc: None,
            committed: false,
        }
    }
}

impl<P, F, I> Parser<I> for Reduce<P, F>
where
    P: StreamedParser<I>,
    F: FnMut(P::Item, P::Item) -> P::Item,
    I: Positioned + ?Sized,
{
    type Output = Option<P::Item>;
    type State = ReduceState<P::State, P::Item>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        loop {
            match ready!(self.inner.poll_parse_next(
                input.as_mut(),
                cx,
                &mut state.inner,
                tracker
            )?) {
                (Some(val), committed) => {
                    state.committed |= committed;
                    match mem::take(&mut state.acc) {
                        Some(acc) => state.acc = Some((self.f)(acc, val)),
                        None => state.acc = Some(val),
                    }
                }
                (None, committed) => {
                    break Poll::Ready(Ok((
                        mem::take(&mut state.acc),
                        state.committed || committed,
                    )))
                }
            }
        }
    }
}

/// A parser for method [`reduce`].
///
/// [`reduce`]: super::StreamedParserExt::reduce
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TryReduce<P, F> {
    inner: P,
    f: F,
}

impl<P, F> TryReduce<P, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, F, E, I> Parser<I> for TryReduce<P, F>
where
    P: StreamedParser<I>,
    F: FnMut(P::Item, P::Item) -> Result<P::Item, E>,
    E: Into<Expects<I::Ok>>,
    I: Positioned + ?Sized,
{
    type Output = Option<P::Item>;
    type State = SpanState<ReduceState<P::State, P::Item>, I::Locator>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        loop {
            state.set_start(|| input.position());
            match ready!(self.inner.poll_parse_next(
                input.as_mut(),
                cx,
                &mut state.inner.inner,
                tracker
            )?) {
                (Some(val), committed) => match mem::take(&mut state.inner.acc) {
                    Some(acc) => match (self.f)(acc, val) {
                        Ok(x) => {
                            state.inner.acc = Some(x);
                            state.inner.committed |= committed;
                            state.start = None;
                        }
                        Err(err) => {
                            tracker.clear();
                            break Poll::Ready(Err(ParseError::Parser {
                                expects: err.into(),
                                position: state.take_start()..input.position(),
                                fatal: true,
                            }));
                        }
                    },
                    None => {
                        state.inner.acc = Some(val);
                        state.inner.committed |= committed;
                    }
                },
                (None, committed) => {
                    break Poll::Ready(Ok((
                        mem::take(&mut state.inner.acc),
                        state.inner.committed || committed,
                    )))
                }
            }
        }
    }
}
