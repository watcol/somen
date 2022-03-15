use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::utils::merge_errors;
use crate::prelude::{Positioned, StreamedParser};

/// A streamed parser generated from method [`flat_times`].
///
/// [`flat_times`]: crate::parser::streamed::StreamedParserExt::flat_times
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatTimes<P> {
    inner: P,
    count: usize,
}

impl<P> FlatTimes<P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, count: usize) -> Self {
        Self { inner, count }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct FlatTimesState<I, P: StreamedParser> {
        inner: P::State,
        count: usize,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, I> StreamedParser<I> for FlatTimes<P>
where
    P: StreamedParser<I>,
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
                break (
                    Status::Success(None, state.error()),
                    state.start()..input.position(),
                );
            }

            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
            {
                (Status::Success(Some(val), err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                    break (
                        Status::Success(Some(val), state.error()),
                        state.start()..pos.end,
                    );
                }
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                    state.count += 1;
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err), &pos);
                    state.set_start(|| pos.start);
                    break (
                        Status::Failure(state.error().unwrap(), false),
                        state.start()..pos.end,
                    );
                }
                (Status::Failure(err, true), pos) => {
                    state.set_start(|| pos.start);
                    break (Status::Failure(err, true), state.start()..pos.end);
                }
            }
        }))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}
