use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Expects, ParseError, PolledResult, Tracker};
use crate::parser::utils::{EitherState, SpanState};
use crate::parser::Parser;
use crate::stream::Positioned;

use super::StreamedParser;

/// A parser for method [`fold`].
///
/// [`fold`]: super::StreamedParserExt::fold
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fold<P, Q, F> {
    inner: P,
    init: Q,
    f: F,
}

impl<P, Q, F> Fold<P, Q, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, init: Q, f: F) -> Self {
        Self { inner, init, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoldState<C, D, T> {
    inner: EitherState<C, D>,
    acc: Option<T>,
    committed: bool,
}

impl<C: Default, D, T> Default for FoldState<C, D, T> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::default(),
            acc: None,
            committed: false,
        }
    }
}

impl<P, F, Q, I> Parser<I> for Fold<P, Q, F>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    F: FnMut(Q::Output, P::Item) -> Q::Output,
    I: Positioned + ?Sized,
{
    type Output = Q::Output;
    type State = FoldState<Q::State, P::State, Q::Output>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        if state.acc.is_none() {
            let (output, committed) = ready!(self.init.poll_parse(
                input.as_mut(),
                cx,
                state.inner.as_mut_left(),
                tracker
            ))?;
            state.committed = committed;
            state.acc = Some(output);
            state.inner = EitherState::Right(Default::default());
        }

        loop {
            match ready!(self.inner.poll_parse_next(
                input.as_mut(),
                cx,
                state.inner.as_mut_right(),
                tracker
            )) {
                Ok((Some(val), committed)) => {
                    state.committed |= committed;
                    state.acc = Some((self.f)(mem::take(&mut state.acc).unwrap(), val));
                }
                Ok((None, committed)) => {
                    break Poll::Ready(Ok((
                        mem::take(&mut state.acc).unwrap(),
                        state.committed | committed,
                    )))
                }
                Err(err) => break Poll::Ready(Err(err.fatal_if(state.committed))),
            }
        }
    }
}

/// A parser for method [`try_fold`].
///
/// [`try_fold`]: super::StreamedParserExt::try_fold
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TryFold<P, Q, F> {
    inner: P,
    init: Q,
    f: F,
}

impl<P, Q, F> TryFold<P, Q, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, init: Q, f: F) -> Self {
        Self { inner, init, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

type TryFoldState<C, D, T, L> = SpanState<FoldState<C, D, T>, L>;
impl<P, Q, F, E, I> Parser<I> for TryFold<P, Q, F>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    F: FnMut(Q::Output, P::Item) -> Result<Q::Output, E>,
    E: Into<Expects<I::Ok>>,
    I: Positioned + ?Sized,
{
    type Output = Q::Output;
    type State = TryFoldState<Q::State, P::State, Q::Output, I::Locator>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        if state.inner.acc.is_none() {
            let (output, committed) = ready!(self.init.poll_parse(
                input.as_mut(),
                cx,
                state.inner.inner.as_mut_left(),
                tracker
            ))?;
            state.inner.acc = Some(output);
            state.inner.inner = EitherState::Right(Default::default());
            state.inner.committed = committed;
        }

        loop {
            state.set_start(|| input.position());
            match ready!(self.inner.poll_parse_next(
                input.as_mut(),
                cx,
                state.inner.inner.as_mut_right(),
                tracker
            )) {
                Ok((Some(val), committed)) => {
                    match (self.f)(mem::take(&mut state.inner.acc).unwrap(), val) {
                        Ok(x) => {
                            state.inner.committed |= committed;
                            state.inner.acc = Some(x);
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
                    }
                }
                Ok((None, committed)) => {
                    break Poll::Ready(Ok((
                        mem::take(&mut state.inner.acc).unwrap(),
                        state.inner.committed || committed,
                    )))
                }
                Err(err) => break Poll::Ready(Err(err.fatal_if(state.inner.committed))),
            }
        }
    }
}
