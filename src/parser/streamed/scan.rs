use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Expects, ParseError, PolledResult, Tracker};
use crate::parser::utils::{EitherState, SpanState};
use crate::parser::Parser;
use crate::stream::Positioned;

use super::StreamedParser;

/// A parser for method [`scan`].
///
/// [`scan`]: super::StreamedParserExt::scan
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scan<P, Q, F> {
    inner: P,
    init: Q,
    f: F,
}

impl<P, Q, F> Scan<P, Q, F> {
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
pub struct ScanState<C, D, T> {
    inner: EitherState<C, D>,
    state: Option<T>,
    committed: bool,
}

impl<C: Default, D, T> Default for ScanState<C, D, T> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::default(),
            state: None,
            committed: false,
        }
    }
}

impl<P, Q, F, T, I> StreamedParser<I> for Scan<P, Q, F>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    F: FnMut(&mut Q::Output, P::Item) -> Option<T>,
    I: Positioned + ?Sized,
{
    type Item = T;
    type State = ScanState<Q::State, P::State, Q::Output>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        if state.state.is_none() {
            let (output, committed) = ready!(self.init.poll_parse(
                input.as_mut(),
                cx,
                state.inner.as_mut_left(),
                tracker
            ))?;
            state.state = Some(output);
            state.committed = committed;
            state.inner = EitherState::Right(Default::default());
        }

        loop {
            match ready!(self.inner.poll_parse_next(
                input.as_mut(),
                cx,
                state.inner.as_mut_right(),
                tracker
            )) {
                Ok((Some(val), committed)) => match (self.f)(state.state.as_mut().unwrap(), val) {
                    Some(res) => {
                        break Poll::Ready(Ok((
                            Some(res),
                            mem::take(&mut state.committed) || committed,
                        )))
                    }
                    None => continue,
                },
                Ok((None, committed)) => {
                    break Poll::Ready(Ok((None, mem::take(&mut state.committed) || committed)))
                }
                Err(err) => break Poll::Ready(Err(err.fatal_if(state.committed))),
            }
        }
    }
}

/// A parser for method [`try_scan`].
///
/// [`try_scan`]: super::StreamedParserExt::try_scan
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TryScan<P, Q, F> {
    inner: P,
    init: Q,
    f: F,
}

impl<P, Q, F> TryScan<P, Q, F> {
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

type TryFoldState<C, D, T, L> = SpanState<ScanState<C, D, T>, L>;
impl<P, Q, F, T, E, I> StreamedParser<I> for TryScan<P, Q, F>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    F: FnMut(&mut Q::Output, P::Item) -> Result<Option<T>, E>,
    E: Into<Expects<I::Ok>>,
    I: Positioned + ?Sized,
{
    type Item = T;
    type State = TryFoldState<Q::State, P::State, Q::Output, I::Locator>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        if state.inner.state.is_none() {
            let (output, committed) = ready!(self.init.poll_parse(
                input.as_mut(),
                cx,
                state.inner.inner.as_mut_left(),
                tracker
            ))?;
            state.inner.state = Some(output);
            state.inner.committed = committed;
            state.inner.inner = EitherState::Right(Default::default());
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
                    match (self.f)(state.inner.state.as_mut().unwrap(), val) {
                        Ok(Some(res)) => {
                            state.start = None;
                            break Poll::Ready(Ok((
                                Some(res),
                                mem::take(&mut state.inner.committed) || committed,
                            )));
                        }
                        Ok(None) => {
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
                        None,
                        mem::take(&mut state.inner.committed) || committed,
                    )))
                }
                Err(err) => break Poll::Ready(Err(err.fatal_if(state.inner.committed))),
            }
        }
    }
}