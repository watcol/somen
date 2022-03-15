use core::ops::{Bound, RangeBounds};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::prelude::StreamedParser;
use crate::stream::Input;

/// A streamed parser generated from method [`flat_sep_by_end`].
///
/// [`flat_sep_by_end`]: crate::parser::streamed::StreamedParserExt::flat_sep_by_end
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

crate::parser_state! {
    pub struct FlatSepByEndState<I: Input, P: StreamedParser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
        streaming: bool,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
        count: usize,
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
    type State = FlatSepByEndState<I, P, Q>;

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
                    .poll_parse_next(input.as_mut(), cx, state.inner.left())?)
                {
                    (Status::Success(None, err), pos) => {
                        state.streaming = false;
                        state.inner = EitherState::new_right();
                        state.error = err;
                        state.start = Some(pos.start);
                    }
                    res => break res,
                }
            }

            state.set_marker(|| input.as_mut().mark())?;

            if let EitherState::Left(inner) = &mut state.inner {
                match ready!(self.inner.poll_parse_next(input.as_mut(), cx, inner)?) {
                    (Status::Success(Some(val), err), pos) => {
                        input.drop_marker(state.marker())?;
                        state.streaming = true;
                        merge_errors(&mut state.error, err, &pos);
                        state.set_start(|| pos.start);
                        break (
                            Status::Success(Some(val), state.error()),
                            state.start()..pos.end,
                        );
                    }
                    (Status::Success(None, err), pos) => {
                        input.as_mut().drop_marker(state.marker())?;
                        state.set_marker(|| input.as_mut().mark())?;
                        state.inner = EitherState::new_right();
                        merge_errors(&mut state.error, err, &pos);
                        state.set_start(|| pos.start);
                    }
                    (Status::Failure(err, false), pos) if err.rewindable(&pos.start) => {
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
            }

            match ready!(self
                .sep
                .poll_parse(input.as_mut(), cx, state.inner.right())?)
            {
                (Status::Success(_, err), pos) => {
                    input.as_mut().drop_marker(state.marker())?;
                    state.count += 1;
                    state.inner = EitherState::new_left();
                    merge_errors(&mut state.error, err, &pos);
                }
                (Status::Failure(err, false), pos)
                    if err.rewindable(&pos.start) && self.range.contains(&(state.count + 1)) =>
                {
                    input.rewind(state.marker())?;
                    merge_errors(
                        &mut state.error,
                        Some(err),
                        &(pos.start.clone()..pos.start.clone()),
                    );
                    break (
                        Status::Success(None, state.error()),
                        state.start()..pos.start,
                    );
                }
                (Status::Failure(err, false), pos) => {
                    input.drop_marker(state.marker())?;
                    merge_errors(&mut state.error, Some(err), &pos);
                    break (
                        Status::Failure(state.error().unwrap(), false),
                        state.start()..pos.end,
                    );
                }
                (Status::Failure(err, true), pos) => {
                    input.drop_marker(state.marker())?;
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
