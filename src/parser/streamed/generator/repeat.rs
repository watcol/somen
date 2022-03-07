use core::mem;
use core::ops::{Bound, RangeBounds};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{PolledResult, Status};
use crate::parser::Parser;
use crate::prelude::StreamedParser;
use crate::stream::Input;

/// A streamed parser generated from method [`repeat`].
///
/// [`repeat`]: crate::parser::ParserExt::repeat
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repeat<P, R> {
    inner: P,
    range: R,
}

impl<P, R> Repeat<P, R> {
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
    #[derive(PartialEq, Eq)]
    pub struct RepeatState<I: Input, P: Parser> {
        inner: P::State,
        marker: Option<I::Marker>,
        count: usize,
    }
}

impl<P, R, I> StreamedParser<I> for Repeat<P, R>
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
            let pos = input.position();
            return Poll::Ready(Ok((Status::Success(None, None), pos.clone()..pos)));
        }

        // Reserve the marker.
        if state.marker.is_none() {
            state.marker = Some(input.as_mut().mark()?);
        }

        Poll::Ready(Ok(
            match ready!(self
                .inner
                .poll_parse(input.as_mut(), cx, &mut state.inner)?)
            {
                (Status::Success(val, err), pos) => {
                    input.drop_marker(mem::take(&mut state.marker).unwrap())?;
                    state.count += 1;
                    (Status::Success(Some(val), err), pos)
                }
                // Return `None` if `count` already satisfies the minimal bound.
                (Status::Failure(err, false), pos)
                    if err.rewindable(&pos.start) && self.range.contains(&state.count) =>
                {
                    input.rewind(mem::take(&mut state.marker).unwrap())?;
                    (
                        Status::Success(None, Some(err)),
                        pos.start.clone()..pos.start,
                    )
                }
                (Status::Failure(err, exclusive), pos) => {
                    input.drop_marker(mem::take(&mut state.marker).unwrap())?;
                    (Status::Failure(err, exclusive), pos)
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
