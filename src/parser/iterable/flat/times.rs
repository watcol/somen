use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::utils::merge_errors;
use crate::prelude::{Positioned, IterableParser};

/// A iterable parser generated from method [`flat_times`].
///
/// [`flat_times`]: crate::parser::iterable::IterableParserExt::flat_times
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatTimes<P> {
    inner: P,
    count: usize,
}

impl<P> FlatTimes<P> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, count: usize) -> Self {
        Self { inner, count }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct FlatTimesState<I, P: IterableParser> {
        inner: P::State,
        count: usize,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, I> IterableParser<I> for FlatTimes<P>
where
    P: IterableParser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = FlatTimesState<I, P>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        Poll::Ready(Ok(loop {
            if state.count >= self.count {
                break Status::Success(None, state.error());
            }

            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
            {
                Status::Success(Some(val), err) => {
                    merge_errors(&mut state.error, err);
                    break Status::Success(Some(val), state.error());
                }
                Status::Success(None, err) => {
                    merge_errors(&mut state.error, err);
                    state.inner = Default::default();
                    state.count += 1;
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    break Status::Failure(state.error().unwrap(), false);
                }
                Status::Failure(err, true) => break Status::Failure(err, true),
            }
        }))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}
