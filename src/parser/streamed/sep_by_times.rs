use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, PolledResult, Tracker};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::EitherState;
use crate::parser::Parser;
use crate::stream::{Input, Positioned};

/// A parser for method [`flat_sep_by_times`].
///
/// [`flat_sep_by_times`]: super::StreamedParserExt::flat_sep_by_times
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatSepByTimes<P, Q> {
    inner: P,
    sep: Q,
    count: usize,
}

impl<P, Q> FlatSepByTimes<P, Q> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, sep: Q, count: usize) -> Self {
        Self { inner, sep, count }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatSepByTimesState<C, D> {
    inner: EitherState<C, D>,
    count: usize,
    committed: bool,
    prev_committed: bool,
    stream_committed: bool,
}

impl<C: Default, D> Default for FlatSepByTimesState<C, D> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::default(),
            count: 0,
            committed: false,
            prev_committed: false,
            stream_committed: false,
        }
    }
}

impl<P, Q, I> StreamedParser<I> for FlatSepByTimes<P, Q>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = FlatSepByTimesState<P::State, Q::State>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        loop {
            if let EitherState::Right(inner) = &mut state.inner {
                match ready!(self.sep.poll_parse(input.as_mut(), cx, inner, tracker)) {
                    Ok((_, committed)) => {
                        state.committed |= committed;
                        state.prev_committed |= committed;
                        state.inner = EitherState::Left(Default::default());
                    }
                    Err(err) => break Poll::Ready(Err(err.fatal_if(state.committed))),
                }
            }

            match ready!(self.inner.poll_parse_next(
                input.as_mut(),
                cx,
                state.inner.as_mut_left(),
                tracker
            )) {
                Ok((Some(val), committed)) => {
                    state.stream_committed |= committed;
                    break Poll::Ready(Ok((
                        Some(val),
                        mem::take(&mut state.prev_committed) || committed,
                    )));
                }
                Ok((None, committed)) => {
                    state.committed |= mem::take(&mut state.stream_committed) || committed;
                    state.count += 1;
                    if state.count == self.count {
                        break Poll::Ready(Ok((None, committed)));
                    } else {
                        state.prev_committed = committed;
                        state.inner = EitherState::Right(Default::default());
                    }
                }
                Err(err) => break Poll::Ready(Err(err.fatal_if(state.committed))),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner = self.inner.size_hint();
        (inner.0 * self.count, inner.1.map(|x| x * self.count))
    }
}

/// A parser for method [`flat_sep_by_end_times`].
///
/// [`flat_sep_by_end_times`]: super::StreamedParserExt::flat_sep_by_end_times
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatSepByEndTimes<P, Q> {
    inner: P,
    sep: Q,
    count: usize,
}

impl<P, Q> FlatSepByEndTimes<P, Q> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, sep: Q, count: usize) -> Self {
        Self { inner, sep, count }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatSepByEndTimesState<C, D, M> {
    inner: EitherState<C, D>,
    queued_marker: Option<M>,
    count: usize,
    committed: bool,
    prev_committed: bool,
    stream_committed: bool,
}

impl<C: Default, D, M> Default for FlatSepByEndTimesState<C, D, M> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::default(),
            queued_marker: None,
            count: 0,
            committed: false,
            prev_committed: false,
            stream_committed: false,
        }
    }
}

impl<P, Q, I> StreamedParser<I> for FlatSepByEndTimes<P, Q>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    I: Input + ?Sized,
{
    type Item = P::Item;
    type State = FlatSepByEndTimesState<P::State, Q::State, I::Marker>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        loop {
            if state.count == self.count {
                if state.queued_marker.is_none() {
                    state.queued_marker = Some(input.as_mut().mark()?);
                }

                match ready!(self.sep.poll_parse(
                    input.as_mut(),
                    cx,
                    state.inner.as_mut_right(),
                    tracker
                )) {
                    Ok((_, committed)) => {
                        input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                        break Poll::Ready(Ok((None, state.prev_committed || committed)));
                    }
                    Err(ParseError::Parser {
                        expects,
                        fatal: false,
                        ..
                    }) => {
                        input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                        tracker.add(expects);
                        break Poll::Ready(Ok((None, state.prev_committed)));
                    }
                    Err(err) => break Poll::Ready(Err(err)),
                }
            }

            if let EitherState::Right(inner) = &mut state.inner {
                match ready!(self.sep.poll_parse(input.as_mut(), cx, inner, tracker)) {
                    Ok((_, committed)) => {
                        state.committed |= committed;
                        state.prev_committed |= committed;
                        state.inner = EitherState::Left(Default::default());
                    }
                    Err(err) => break Poll::Ready(Err(err.fatal_if(state.committed))),
                }
            }

            match ready!(self.inner.poll_parse_next(
                input.as_mut(),
                cx,
                state.inner.as_mut_left(),
                tracker
            )) {
                Ok((Some(val), committed)) => {
                    state.stream_committed |= committed;
                    break Poll::Ready(Ok((
                        Some(val),
                        mem::take(&mut state.prev_committed) || committed,
                    )));
                }
                Ok((None, committed)) => {
                    state.committed |= mem::take(&mut state.stream_committed) || committed;
                    state.prev_committed |= committed;
                    state.count += 1;
                    state.inner = EitherState::Right(Default::default());
                }
                Err(err) => break Poll::Ready(Err(err.fatal_if(state.committed))),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner = self.inner.size_hint();
        (inner.0 * self.count, inner.1.map(|x| x * self.count))
    }
}
