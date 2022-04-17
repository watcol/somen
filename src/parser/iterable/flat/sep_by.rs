use core::ops::{Bound, RangeBounds};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::prelude::IterableParser;
use crate::stream::Input;

/// A iterable parser generated from method [`flat_sep_by`].
///
/// [`flat_sep_by`]: crate::parser::iterable::IterableParserExt::flat_sep_by
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatSepBy<P, Q, R> {
    inner: P,
    sep: Q,
    range: R,
}

impl<P, Q, R> FlatSepBy<P, Q, R> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, sep: Q, range: R) -> Self {
        Self { inner, sep, range }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct FlatSepByState<I: Input, P: IterableParser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
        count: usize,
        streaming: bool,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Locator>>,
    }
}

impl<P, Q, R, I> IterableParser<I> for FlatSepBy<P, Q, R>
where
    P: IterableParser<I>,
    Q: Parser<I>,
    R: RangeBounds<usize>,
    I: Input + ?Sized,
{
    type Item = P::Item;
    type State = FlatSepByState<I, P, Q>;

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
                break Status::Success(None, state.error());
            } else if state.streaming {
                match ready!(self
                    .inner
                    .poll_parse_next(input.as_mut(), cx, state.inner.left())?)
                {
                    Status::Success(None, err) => {
                        state.streaming = false;
                        state.error = err;
                        state.count += 1;
                        state.inner = EitherState::new_right();
                    }
                    res => break res,
                }
            }

            state.set_marker(|| input.as_mut().mark())?;
            state.set_start(|| input.position());

            if let EitherState::Right(inner) = &mut state.inner {
                match ready!(self.sep.poll_parse(input.as_mut(), cx, inner)?) {
                    Status::Success(_, err) => {
                        state.inner = EitherState::new_left();
                        merge_errors(&mut state.error, err);
                    }
                    Status::Failure(err, false) if err.rewindable(&state.start()) => {
                        input.rewind(state.marker())?;
                        merge_errors(&mut state.error, Some(err));
                        return Poll::Ready(Ok(Status::Success(None, state.error())));
                    }
                    Status::Failure(err, false) => {
                        input.drop_marker(state.marker())?;
                        merge_errors(&mut state.error, Some(err));
                        return Poll::Ready(Ok(Status::Failure(state.error().unwrap(), false)));
                    }
                    Status::Failure(err, true) => {
                        input.drop_marker(state.marker())?;
                        return Poll::Ready(Ok(Status::Failure(err, true)));
                    }
                }
            }

            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, state.inner.left())?)
            {
                Status::Success(Some(val), err) => {
                    input.drop_marker(state.marker())?;
                    state.start = None;
                    state.streaming = true;
                    merge_errors(&mut state.error, err);
                    break Status::Success(Some(val), state.error());
                }
                Status::Success(None, err) => {
                    input.as_mut().drop_marker(state.marker())?;
                    state.start = None;
                    merge_errors(&mut state.error, err);
                    state.count += 1;
                    state.inner = EitherState::new_right();
                }
                Status::Failure(err, false)
                    if err.rewindable(&state.start()) && self.range.contains(&state.count) =>
                {
                    input.rewind(state.marker())?;
                    merge_errors(&mut state.error, Some(err));
                    break Status::Success(None, state.error());
                }
                Status::Failure(err, false) => {
                    input.drop_marker(state.marker())?;
                    merge_errors(&mut state.error, Some(err));
                    break Status::Failure(state.error().unwrap(), false);
                }
                Status::Failure(err, true) => {
                    input.drop_marker(state.marker())?;
                    break Status::Failure(err, true);
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
