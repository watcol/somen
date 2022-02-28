use core::mem;
use core::ops::{Bound, RangeBounds};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, PolledResult, Tracker};
use crate::parser::Parser;
use crate::prelude::StreamedParser;
use crate::stream::Input;

use super::utils::EitherState;

/// A streamed parser generated from method [`sep_by`].
///
/// [`sep_by`]: super::ParserExt::sep_by
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SepBy<P, Q, R> {
    inner: P,
    sep: Q,
    range: R,
}

impl<P, Q, R> SepBy<P, Q, R> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, sep: Q, range: R) -> Self {
        Self { inner, sep, range }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SepByState<C, D, M> {
    inner: EitherState<C, D>,
    queued_marker: Option<M>,
    count: usize,
    committed: bool,
    prev_committed: bool,
}

impl<C: Default, D, M> Default for SepByState<C, D, M> {
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

impl<P, Q, R, I> StreamedParser<I> for SepBy<P, Q, R>
where
    P: Parser<I>,
    Q: Parser<I>,
    R: RangeBounds<usize>,
    I: Input + ?Sized,
{
    type Item = P::Output;
    type State = SepByState<P::State, Q::State, I::Marker>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        // Return `None` if the number of items already reached `end_bound`.
        if match self.range.end_bound() {
            Bound::Included(i) => state.count + 1 > *i,
            Bound::Excluded(i) => state.count + 1 >= *i,
            Bound::Unbounded => false,
        } {
            return Poll::Ready(Ok((None, false)));
        }

        // Reserve the marker.
        if state.queued_marker.is_none() {
            state.queued_marker = Some(input.as_mut().mark()?);
        }

        if let EitherState::Right(inner) = &mut state.inner {
            match ready!(self.sep.poll_parse(input.as_mut(), cx, inner, tracker)) {
                Ok((_, committed)) => {
                    state.inner = EitherState::Left(Default::default());
                    state.prev_committed = committed;
                }
                Err(ParseError::Parser {
                    expects,
                    fatal: false,
                    ..
                }) if self.range.contains(&state.count) => {
                    input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    tracker.add(expects);
                    return Poll::Ready(Ok((None, false)));
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
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
                    input
                        .as_mut()
                        .drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    let committed = committed || state.prev_committed;
                    state.committed |= committed;
                    state.prev_committed = false;
                    state.inner = EitherState::Right(Default::default());
                    state.count += 1;
                    Ok((Some(output), committed))
                }
                // Return `None` if `count` already satisfies the minimal bound.
                Err(ParseError::Parser {
                    fatal: false,
                    expects,
                    ..
                }) if self.range.contains(&state.count) && state.count == 0 => {
                    input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    tracker.add(expects);
                    Ok((None, false))
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    // If the parser has succeeded parsing at least once, rewinding the parser is
                    // not appropriate.
                    Err(err.fatal_if(state.committed || state.prev_committed))
                }
            },
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let start = match self.range.start_bound() {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => *i + 1,
            Bound::Unbounded => 0,
        };

        let end = match self.range.end_bound() {
            Bound::Included(i) => Some(*i),
            Bound::Excluded(i) => Some(*i - 1),
            Bound::Unbounded => None,
        };

        (start, end)
    }
}

/// A streamed parser generated from method [`sep_by_end`].
///
/// [`sep_by_end`]: super::ParserExt::sep_by_end
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SepByEnd<P, Q, R> {
    inner: P,
    sep: Q,
    range: R,
}

impl<P, Q, R> SepByEnd<P, Q, R> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, sep: Q, range: R) -> Self {
        Self { inner, sep, range }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SepByEndState<C, D, O, M> {
    inner: EitherState<C, D>,
    output: Option<O>,
    queued_marker: Option<M>,
    count: usize,
    ended: bool,
    committed: bool,
    prev_committed: bool,
}

impl<C: Default, D, O, M> Default for SepByEndState<C, D, O, M> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::default(),
            output: None,
            queued_marker: None,
            count: 0,
            ended: false,
            committed: false,
            prev_committed: false,
        }
    }
}

impl<P, Q, R, I> StreamedParser<I> for SepByEnd<P, Q, R>
where
    P: Parser<I>,
    Q: Parser<I>,
    R: RangeBounds<usize>,
    I: Input + ?Sized,
{
    type Item = P::Output;
    type State = SepByEndState<P::State, Q::State, P::Output, I::Marker>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        // Return `None` if the number of items already reached `end_bound`.
        if match self.range.end_bound() {
            Bound::Included(i) => state.count + 1 > *i,
            Bound::Excluded(i) => state.count + 1 >= *i,
            Bound::Unbounded => false,
        } || state.ended
        {
            return Poll::Ready(Ok((None, false)));
        }

        // Reserve the marker.
        if state.queued_marker.is_none() {
            state.queued_marker = Some(input.as_mut().mark()?);
        }

        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.inner.poll_parse(input.as_mut(), cx, inner, tracker)) {
                Ok((output, committed)) => {
                    input
                        .as_mut()
                        .drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    state.inner = EitherState::Right(Default::default());
                    state.queued_marker = Some(input.as_mut().mark()?);
                    state.output = Some(output);
                    state.prev_committed = committed;
                }
                // Return `None` if `count` already satisfies the minimal bound.
                Err(ParseError::Parser {
                    fatal: false,
                    expects,
                    ..
                }) if self.range.contains(&state.count) => {
                    input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    tracker.add(expects);
                    return Poll::Ready(Ok((None, false)));
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    // If the parser has succeeded parsing at least once, rewinding the parser is
                    // not appropriate.
                    return Poll::Ready(Err(err.fatal_if(state.committed)));
                }
            }
        }

        Poll::Ready(
            match ready!(self.sep.poll_parse(
                input.as_mut(),
                cx,
                state.inner.as_mut_right(),
                tracker
            )) {
                Ok((_, committed)) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    let committed = state.prev_committed || committed;
                    state.committed |= true;
                    state.prev_committed = false;
                    state.count += 1;
                    state.inner = EitherState::Left(Default::default());
                    Ok((Some(mem::take(&mut state.output).unwrap()), committed))
                }
                Err(ParseError::Parser {
                    fatal: false,
                    expects,
                    ..
                }) if self.range.contains(&(state.count + 1)) => {
                    input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    let committed = state.prev_committed;
                    tracker.add(expects);
                    state.count += 1;
                    state.ended = true;
                    state.prev_committed = false;
                    Ok((Some(mem::take(&mut state.output).unwrap()), committed))
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    Err(err.fatal_if(state.committed || state.prev_committed))
                }
            },
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let start = match self.range.start_bound() {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => *i + 1,
            Bound::Unbounded => 0,
        };

        let end = match self.range.end_bound() {
            Bound::Included(i) => Some(*i),
            Bound::Excluded(i) => Some(*i - 1),
            Bound::Unbounded => None,
        };

        (start, end)
    }
}
