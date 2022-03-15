use core::ops::{Bound, RangeBounds};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::prelude::StreamedParser;
use crate::stream::Input;

/// A streamed parser generated from method [`sep_by`].
///
/// [`sep_by`]: crate::parser::ParserExt::sep_by
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

crate::parser_state! {
    pub struct SepByState<I: Input, P: Parser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
        count: usize,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
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
            let pos = input.position();
            return Poll::Ready(Ok((Status::Success(None, None), pos.clone()..pos)));
        }

        state.set_marker(|| input.as_mut().mark())?;

        if let EitherState::Right(inner) = &mut state.inner {
            match ready!(self.sep.poll_parse(input.as_mut(), cx, inner)?) {
                (Status::Success(_, err), pos) => {
                    state.inner = EitherState::new_left();
                    state.error = err;
                    state.start = Some(pos.start);
                }
                (Status::Failure(err, false), pos) if err.rewindable(&pos.start) => {
                    input.rewind(state.marker())?;
                    return Poll::Ready(Ok((
                        Status::Success(None, Some(err)),
                        pos.start.clone()..pos.start,
                    )));
                }
                (Status::Failure(err, exclusive), pos) => {
                    input.drop_marker(state.marker())?;
                    return Poll::Ready(Ok((Status::Failure(err, exclusive), pos)));
                }
            }
        }

        Poll::Ready(Ok(
            match ready!(self
                .inner
                .poll_parse(input.as_mut(), cx, state.inner.left())?)
            {
                (Status::Success(val, err), pos) => {
                    input.drop_marker(state.marker())?;
                    state.count += 1;
                    state.inner = EitherState::new_right();
                    merge_errors(&mut state.error, err);
                    state.set_start(|| pos.start);
                    (
                        Status::Success(Some(val), state.error()),
                        state.start()..pos.end,
                    )
                }
                (Status::Failure(err, false), pos)
                    if err.rewindable(&pos.start) && self.range.contains(&state.count) =>
                {
                    input.rewind(state.marker())?;
                    merge_errors(&mut state.error, Some(err));
                    state.set_start(|| pos.start.clone());
                    (
                        Status::Success(None, state.error()),
                        state.start()..pos.start,
                    )
                }
                (Status::Failure(err, false), pos) => {
                    input.drop_marker(state.marker())?;
                    merge_errors(&mut state.error, Some(err));
                    state.set_start(|| pos.start);
                    (
                        Status::Failure(state.error().unwrap(), false),
                        state.start()..pos.end,
                    )
                }
                (Status::Failure(err, true), pos) => {
                    input.drop_marker(state.marker())?;
                    state.set_start(|| pos.start);
                    (Status::Failure(err, true), state.start()..pos.end)
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
