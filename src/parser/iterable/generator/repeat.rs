use core::ops::{Bound, RangeBounds};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{PolledResult, Status};
use crate::parser::Parser;
use crate::prelude::IterableParser;
use crate::stream::Input;

/// A iterable parser generated from method [`repeat`].
///
/// [`repeat`]: crate::parser::ParserExt::repeat
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repeat<P, R> {
    inner: P,
    range: R,
}

impl<P, R> Repeat<P, R> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, range: R) -> Self {
        Self { inner, range }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct RepeatState<I: Input, P: Parser> {
        inner: P::State,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
        #[opt(set = set_start)]
        start: I::Locator,
        count: usize,
    }
}

impl<P, R, I> IterableParser<I> for Repeat<P, R>
where
    P: Parser<I>,
    R: RangeBounds<usize>,
    I: Input + ?Sized,
{
    type Item = P::Output;
    type State = RepeatState<I, P>;

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

        // Reserve the marker.
        state.set_marker(|| input.as_mut().mark())?;
        state.set_start(|| input.position());

        Poll::Ready(Ok(
            match ready!(self
                .inner
                .poll_parse(input.as_mut(), cx, &mut state.inner)?)
            {
                Status::Success(val, err) => {
                    input.drop_marker(state.marker())?;
                    state.start = None;
                    state.inner = Default::default();
                    state.count += 1;
                    Status::Success(Some(val), err)
                }
                // Return `None` if `count` already satisfies the minimal bound.
                Status::Failure(err, false)
                    if err.rewindable(&state.start()) && self.range.contains(&state.count) =>
                {
                    input.rewind(state.marker())?;
                    Status::Success(None, Some(err))
                }
                Status::Failure(err, exclusive) => {
                    input.drop_marker(state.marker())?;
                    Status::Failure(err, exclusive)
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
