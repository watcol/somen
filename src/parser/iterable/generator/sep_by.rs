use core::ops::{Bound, RangeBounds};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::prelude::IterableParser;
use crate::stream::Input;

/// A iterable parser generated from method [`sep_by`].
///
/// [`sep_by`]: crate::parser::ParserExt::sep_by
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SepBy<P, Q, R> {
    inner: P,
    sep: Q,
    range: R,
}

impl<P, Q, R> SepBy<P, Q, R> {
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
    pub struct SepByState<I: Input, P: Parser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
        count: usize,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Locator>>,
    }
}

impl<P, Q, R, I> IterableParser<I> for SepBy<P, Q, R>
where
    P: Parser<I>,
    Q: Parser<I>,
    R: RangeBounds<usize>,
    I: Input + ?Sized,
{
    type Item = P::Output;
    type State = SepByState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        // Return `None` if the number of items already reached `end_bound`.
        if match self.range.end_bound() {
            Bound::Included(i) => state.count + 1 > *i,
            Bound::Excluded(i) => state.count + 1 >= *i,
            Bound::Unbounded => false,
        } {
            return Poll::Ready(Ok(Status::Success(None, None)));
        }

        state.set_marker(|| input.as_mut().mark())?;
        state.set_start(|| input.position());

        if let EitherState::Right(inner) = &mut state.inner {
            match ready!(self.sep.poll_parse(input.as_mut(), cx, inner)?) {
                Status::Success(_, err) => {
                    state.inner = EitherState::new_left();
                    state.error = err;
                }
                Status::Failure(err, false) if err.rewindable(&state.start()) => {
                    input.rewind(state.marker())?;
                    return Poll::Ready(Ok(Status::Success(None, Some(err))));
                }
                Status::Failure(err, exclusive) => {
                    input.drop_marker(state.marker())?;
                    return Poll::Ready(Ok(Status::Failure(err, exclusive)));
                }
            }
        }

        Poll::Ready(Ok(
            match ready!(self
                .inner
                .poll_parse(input.as_mut(), cx, state.inner.left())?)
            {
                Status::Success(val, err) => {
                    input.drop_marker(state.marker())?;
                    state.start = None;
                    state.count += 1;
                    state.inner = EitherState::new_right();
                    merge_errors(&mut state.error, err);
                    Status::Success(Some(val), state.error())
                }
                Status::Failure(err, false)
                    if err.rewindable(&state.start()) && self.range.contains(&state.count) =>
                {
                    input.rewind(state.marker())?;
                    merge_errors(&mut state.error, Some(err));
                    Status::Success(None, state.error())
                }
                Status::Failure(err, false) => {
                    input.drop_marker(state.marker())?;
                    merge_errors(&mut state.error, Some(err));
                    Status::Failure(state.error().unwrap(), false)
                }
                Status::Failure(err, true) => {
                    input.drop_marker(state.marker())?;
                    Status::Failure(err, true)
                }
            },
        ))
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
