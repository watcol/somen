use core::ops::{Bound, RangeBounds};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::utils::merge_errors;
use crate::prelude::StreamedParser;
use crate::stream::Input;

/// A streamed parser generated from method [`flat_repeat`].
///
/// [`flat_repeat`]: crate::parser::streamed::StreamedParserExt::flat_repeat
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

crate::parser_state! {
    pub struct FlatRepeatState<I: Input, P: StreamedParser> {
        inner: P::State,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
        count: usize,
        streaming: bool,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, R, I> StreamedParser<I> for FlatRepeat<P, R>
where
    P: StreamedParser<I>,
    R: RangeBounds<usize>,
    I: Input + ?Sized,
{
    type Item = P::Item;
    type State = FlatRepeatState<I, P>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        Poll::Ready(Ok(loop {
            // Return `None` if the number of items already reached `end_bound`.
            if match self.range.end_bound() {
                Bound::Included(i) => state.count + 1 > *i,
                Bound::Excluded(i) => state.count + 1 >= *i,
                Bound::Unbounded => false,
            } {
                break (
                    Status::Success(None, state.error()),
                    state.start()..input.position(),
                );
            } else if state.streaming {
                match ready!(self
                    .inner
                    .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
                {
                    (Status::Success(None, err), pos) => {
                        state.error = err;
                        state.start = Some(pos.start);
                        state.streaming = false;
                        state.count += 1;
                    }
                    res => break res,
                }
            }

            // Reserve the marker.
            state.set_marker(|| input.as_mut().mark())?;

            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
            {
                (Status::Success(Some(val), err), pos) => {
                    input.drop_marker(state.marker())?;
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                    state.streaming = true;
                    break (
                        Status::Success(Some(val), state.error()),
                        state.start()..pos.end,
                    );
                }
                (Status::Success(None, err), pos) => {
                    input.as_mut().drop_marker(state.marker())?;
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                    state.count += 1;
                }
                // Return `None` if `count` already satisfies the minimal bound.
                (Status::Failure(err, false), pos)
                    if err.rewindable(&pos.start) && self.range.contains(&state.count) =>
                {
                    input.rewind(state.marker())?;
                    merge_errors(
                        &mut state.error,
                        Some(err),
                        &(pos.start.clone()..pos.start.clone()),
                    );
                    state.set_start(|| pos.start.clone());
                    break (
                        Status::Success(None, state.error()),
                        state.start()..pos.start,
                    );
                }
                (Status::Failure(err, false), pos) => {
                    input.drop_marker(state.marker())?;
                    merge_errors(&mut state.error, Some(err), &pos);
                    state.set_start(|| pos.start);
                    break (
                        Status::Failure(state.error().unwrap(), false),
                        state.start()..pos.end,
                    );
                }
                (Status::Failure(err, true), pos) => {
                    input.drop_marker(state.marker())?;
                    state.set_start(|| pos.start);
                    break (Status::Failure(err, true), state.start()..pos.end);
                }
            }
        }))
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