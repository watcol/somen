use core::mem;
use core::ops::{Bound, RangeBounds};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, ParseResult, Tracker};
use crate::parser::utils::EitherState;
use crate::parser::Parser;
use crate::prelude::StreamedParser;
use crate::stream::Input;

/// A parser for method [`flat_sep_by`].
///
/// [`flat_sep_by`]: super::StreamedParserExt::flat_sep_by
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatSepBy<P, Q, R> {
    inner: P,
    sep: Q,
    range: R,
}

impl<P, Q, R> FlatSepBy<P, Q, R> {
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
pub struct FlatSepByState<C, D, M> {
    inner: EitherState<C, D>,
    queued_marker: Option<M>,
    count: usize,
    streaming: bool,
}

impl<C: Default, D, M> Default for FlatSepByState<C, D, M> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::default(),
            queued_marker: None,
            count: 0,
            streaming: false,
        }
    }
}

impl<P, Q, R, I> StreamedParser<I> for FlatSepBy<P, Q, R>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    R: RangeBounds<usize>,
    I: Input + ?Sized,
{
    type Item = P::Item;
    type State = FlatSepByState<P::State, Q::State, I::Marker>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        loop {
            // Return `None` if the number of items already reached `end_bound`.
            if match self.range.end_bound() {
                Bound::Included(i) => state.count + 1 > *i,
                Bound::Excluded(i) => state.count + 1 >= *i,
                Bound::Unbounded => false,
            } {
                break Poll::Ready(Ok(None));
            } else if state.streaming {
                match ready!(self.inner.poll_parse_next(
                    input.as_mut(),
                    cx,
                    state.inner.as_mut_left(),
                    tracker
                )) {
                    Ok(Some(val)) => break Poll::Ready(Ok(Some(val))),
                    Ok(None) => {
                        state.count += 1;
                        state.inner = EitherState::Right(Default::default());
                        state.streaming = false;
                    }
                    Err(err) => break Poll::Ready(Err(err.fatal(true))),
                }
            }

            // Reserve the marker.
            if state.queued_marker.is_none() {
                state.queued_marker = Some(input.as_mut().mark()?);
            }

            if let EitherState::Right(inner) = &mut state.inner {
                match ready!(self.sep.poll_parse(input.as_mut(), cx, inner, tracker)) {
                    Ok(_) => {
                        state.inner = EitherState::Left(Default::default());
                    }
                    Err(ParseError::Parser {
                        expects,
                        fatal: false,
                        ..
                    }) if self.range.contains(&state.count) => {
                        input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                        tracker.add(expects);
                        break Poll::Ready(Ok(None));
                    }
                    Err(err) => {
                        input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                        break Poll::Ready(Err(err.fatal(true)));
                    }
                }
            }

            match ready!(self.inner.poll_parse_next(
                input.as_mut(),
                cx,
                state.inner.as_mut_left(),
                tracker
            )) {
                Ok(Some(val)) => {
                    input
                        .as_mut()
                        .drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    state.streaming = true;
                    break Poll::Ready(Ok(Some(val)));
                }
                Ok(None) => {
                    input
                        .as_mut()
                        .drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    state.count += 1;
                    state.inner = EitherState::Right(Default::default());
                }
                // Return `None` if `count` already satisfies the minimal bound.
                Err(ParseError::Parser {
                    fatal: false,
                    expects,
                    ..
                }) if self.range.contains(&state.count) && state.count == 0 => {
                    input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    tracker.add(expects);
                    break Poll::Ready(Ok(None));
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    // If the parser has succeeded parsing at least once, rewinding the parser is
                    // not appropriate.
                    break Poll::Ready(Err(if state.count > 0 {
                        err.fatal(true)
                    } else {
                        err
                    }));
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner = self.inner.size_hint();
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

        (inner.0 * start, inner.1.zip(end).map(|(a, b)| a * b))
    }
}

/// A parser for method [`flat_sep_by_end`].
///
/// [`flat_sep_by_end`]: super::StreamedParserExt::flat_sep_by_end
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatSepByEnd<P, Q, R> {
    inner: P,
    sep: Q,
    range: R,
}

impl<P, Q, R> FlatSepByEnd<P, Q, R> {
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
pub struct FlatSepByEndState<C, D, M> {
    inner: EitherState<C, D>,
    queued_marker: Option<M>,
    count: usize,
    streaming: bool,
}

impl<C: Default, D, M> Default for FlatSepByEndState<C, D, M> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::default(),
            queued_marker: None,
            count: 0,
            streaming: false,
        }
    }
}

impl<P, Q, R, I> StreamedParser<I> for FlatSepByEnd<P, Q, R>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    R: RangeBounds<usize>,
    I: Input + ?Sized,
{
    type Item = P::Item;
    type State = FlatSepByEndState<P::State, Q::State, I::Marker>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        loop {
            // Return `None` if the number of items already reached `end_bound`.
            if match self.range.end_bound() {
                Bound::Included(i) => state.count + 1 > *i,
                Bound::Excluded(i) => state.count + 1 >= *i,
                Bound::Unbounded => false,
            } {
                break Poll::Ready(Ok(None));
            } else if state.streaming {
                match ready!(self.inner.poll_parse_next(
                    input.as_mut(),
                    cx,
                    state.inner.as_mut_left(),
                    tracker
                )) {
                    Ok(Some(val)) => break Poll::Ready(Ok(Some(val))),
                    Ok(None) => {
                        state.streaming = false;
                        state.inner = EitherState::Right(Default::default());
                    }
                    Err(err) => break Poll::Ready(Err(err.fatal(true))),
                }
            }

            // Reserve the marker.
            if state.queued_marker.is_none() {
                state.queued_marker = Some(input.as_mut().mark()?);
            }

            if let EitherState::Left(inner) = &mut state.inner {
                match ready!(self
                    .inner
                    .poll_parse_next(input.as_mut(), cx, inner, tracker))
                {
                    Ok(Some(val)) => {
                        input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                        state.streaming = true;
                        break Poll::Ready(Ok(Some(val)));
                    }
                    Ok(None) => {
                        input
                            .as_mut()
                            .drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                        state.inner = EitherState::Right(Default::default());
                        state.queued_marker = Some(input.as_mut().mark()?);
                    }
                    // Return `None` if `count` already satisfies the minimal bound.
                    Err(ParseError::Parser {
                        fatal: false,
                        expects,
                        ..
                    }) if self.range.contains(&state.count) => {
                        input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                        tracker.add(expects);
                        break Poll::Ready(Ok(None));
                    }
                    Err(err) => {
                        input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                        // If the parser has succeeded parsing at least once, rewinding the parser is
                        // not appropriate.
                        break Poll::Ready(Err(if state.count > 0 {
                            err.fatal(true)
                        } else {
                            err
                        }));
                    }
                }
            }

            match ready!(self.sep.poll_parse(
                input.as_mut(),
                cx,
                state.inner.as_mut_right(),
                tracker
            )) {
                Ok(_) => {
                    input
                        .as_mut()
                        .drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    state.count += 1;
                    state.inner = EitherState::Left(Default::default());
                }
                Err(ParseError::Parser {
                    fatal: false,
                    expects,
                    ..
                }) if self.range.contains(&(state.count + 1)) => {
                    input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    tracker.add(expects);
                    break Poll::Ready(Ok(None));
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    break Poll::Ready(Err(err.fatal(true)));
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner = self.inner.size_hint();
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

        (inner.0 * start, inner.1.zip(end).map(|(a, b)| a * b))
    }
}
