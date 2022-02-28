use core::mem;
use core::ops::{Bound, RangeBounds};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, PolledResult, Tracker};
use crate::prelude::StreamedParser;
use crate::stream::Input;

/// A parser for method [`flat_repeat`].
///
/// [`flat_repeat`]: super::StreamedParserExt::flat_repeat
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatRepeat<P, R> {
    inner: P,
    range: R,
}

impl<P, R> FlatRepeat<P, R> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, range: R) -> Self {
        Self { inner, range }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatRepeatState<C, M> {
    inner: C,
    queued_marker: Option<M>,
    count: usize,
    streaming: bool,
    committed: bool,
    prev_committed: bool,
}

impl<C: Default, M> Default for FlatRepeatState<C, M> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: C::default(),
            queued_marker: None,
            count: 0,
            streaming: false,
            committed: false,
            prev_committed: false,
        }
    }
}

impl<P, R, I> StreamedParser<I> for FlatRepeat<P, R>
where
    P: StreamedParser<I>,
    R: RangeBounds<usize>,
    I: Input + ?Sized,
{
    type Item = P::Item;
    type State = FlatRepeatState<P::State, I::Marker>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        // Return `None` if the number of items already reached `end_bound`.
        loop {
            if match self.range.end_bound() {
                Bound::Included(i) => state.count + 1 > *i,
                Bound::Excluded(i) => state.count + 1 >= *i,
                Bound::Unbounded => false,
            } {
                break Poll::Ready(Ok((None, state.prev_committed)));
            } else if state.streaming {
                match ready!(self.inner.poll_parse_next(
                    input.as_mut(),
                    cx,
                    &mut state.inner,
                    tracker
                )) {
                    Ok((Some(val), committed)) => {
                        state.prev_committed |= committed;
                        break Poll::Ready(Ok((Some(val), committed)));
                    }
                    Ok((None, committed)) => {
                        state.inner = Default::default();
                        state.count += 1;
                        state.committed |= state.prev_committed || committed;
                        state.prev_committed = committed;
                        state.streaming = false;
                    }
                    Err(err) => break Poll::Ready(Err(err.fatal(state.committed))),
                }
            }

            // Reserve the marker.
            if state.queued_marker.is_none() {
                state.queued_marker = Some(input.as_mut().mark()?);
            }

            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner, tracker))
            {
                Ok((Some(val), committed)) => {
                    input
                        .as_mut()
                        .drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    state.streaming = true;
                    let ret_committed = state.prev_committed || committed;
                    state.prev_committed = committed;
                    break Poll::Ready(Ok((Some(val), ret_committed)));
                }
                Ok((None, committed)) => {
                    input
                        .as_mut()
                        .drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    state.committed |= committed;
                    state.prev_committed |= committed;
                    state.inner = Default::default();
                    state.count += 1;
                }
                // Return `None` if `count` already satisfies the minimal bound.
                Err(ParseError::Parser {
                    fatal: false,
                    expects,
                    ..
                }) if self.range.contains(&state.count) => {
                    input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    tracker.add(expects);
                    break Poll::Ready(Ok((None, false)));
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    break Poll::Ready(Err(err.fatal_if(state.committed)));
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
