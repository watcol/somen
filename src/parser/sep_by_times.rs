use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use super::utils::EitherState;
use crate::error::{ParseError, PolledResult, Tracker};
use crate::parser::streamed::StreamedParser;
use crate::parser::Parser;
use crate::stream::{Input, Positioned};

/// A streamed parser generated from method [`sep_by_times`].
///
/// [`sep_by_times`]: super::ParserExt::sep_by_times
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SepByTimes<P, Q> {
    inner: P,
    sep: Q,
    count: usize,
}

impl<P, Q> SepByTimes<P, Q> {
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
pub struct SepByTimesState<C, D> {
    inner: EitherState<C, D>,
    count: usize,
    committed: bool,
    prev_committed: bool,
}

impl<C: Default, D> Default for SepByTimesState<C, D> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::default(),
            count: 0,
            committed: false,
            prev_committed: false,
        }
    }
}

impl<P, Q, I> StreamedParser<I> for SepByTimes<P, Q>
where
    P: Parser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Output;
    type State = SepByTimesState<P::State, Q::State>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        if state.count == self.count {
            return Poll::Ready(Ok((None, false)));
        }

        if let EitherState::Right(inner) = &mut state.inner {
            match ready!(self.sep.poll_parse(input.as_mut(), cx, inner, tracker)) {
                Ok((_, committed)) => {
                    state.prev_committed = committed;
                    state.inner = EitherState::Left(Default::default());
                }
                Err(err) => {
                    return Poll::Ready(Err(err.fatal_if(state.committed)));
                }
            }
        }

        Poll::Ready(
            match ready!(self.inner.poll_parse(
                input.as_mut(),
                cx,
                state.inner.as_mut_left(),
                tracker
            )) {
                Ok((output, committed)) => {
                    let committed = state.prev_committed || committed;
                    state.committed |= committed;
                    state.prev_committed = false;
                    state.inner = EitherState::Right(Default::default());
                    state.count += 1;
                    Ok((Some(output), committed))
                }
                Err(err) => Err(err.fatal_if(state.committed || state.prev_committed)),
            },
        )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

/// A streamed parser generated from method [`sep_by_end_times`].
///
/// [`sep_by_end_times`]: super::ParserExt::sep_by_end_times
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SepByEndTimes<P, Q> {
    inner: P,
    sep: Q,
    count: usize,
}

impl<P, Q> SepByEndTimes<P, Q> {
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
pub struct SepByEndTimesState<C, D, M> {
    inner: EitherState<C, D>,
    queued_marker: Option<M>,
    count: usize,
    committed: bool,
    prev_committed: bool,
}

impl<C: Default, D, M> Default for SepByEndTimesState<C, D, M> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::default(),
            queued_marker: None,
            count: 0,
            committed: false,
            prev_committed: false,
        }
    }
}

impl<P, Q, I> StreamedParser<I> for SepByEndTimes<P, Q>
where
    P: Parser<I>,
    Q: Parser<I>,
    I: Input + ?Sized,
{
    type Item = P::Output;
    type State = SepByEndTimesState<P::State, Q::State, I::Marker>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        if state.count == self.count {
            if state.queued_marker.is_none() {
                state.queued_marker = Some(input.as_mut().mark()?);
            }

            return Poll::Ready(
                match ready!(self.sep.poll_parse(
                    input.as_mut(),
                    cx,
                    state.inner.as_mut_right(),
                    tracker
                )) {
                    Ok((_, committed)) => {
                        input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                        Ok((None, committed))
                    }
                    Err(ParseError::Parser {
                        expects,
                        fatal: false,
                        ..
                    }) => {
                        input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                        tracker.add(expects);
                        Ok((None, false))
                    }
                    Err(err) => Err(err),
                },
            );
        }

        if let EitherState::Right(inner) = &mut state.inner {
            match ready!(self.sep.poll_parse(input.as_mut(), cx, inner, tracker)) {
                Ok((_, committed)) => {
                    state.prev_committed |= committed;
                    state.inner = EitherState::Left(Default::default());
                }
                Err(err) => return Poll::Ready(Err(err.fatal_if(state.committed))),
            }
        }

        Poll::Ready(
            match ready!(self.inner.poll_parse(
                input.as_mut(),
                cx,
                state.inner.as_mut_left(),
                tracker
            )) {
                Ok((output, committed)) => {
                    let committed = state.prev_committed || committed;
                    state.committed |= committed;
                    state.prev_committed = false;
                    state.inner = EitherState::Right(Default::default());
                    state.count += 1;
                    Ok((Some(output), committed))
                }
                Err(err) => Err(err.fatal_if(state.committed || state.prev_committed)),
            },
        )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}
