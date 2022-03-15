use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A streamed parser generated from method [`flat_sep_by_times`].
///
/// [`flat_sep_by_times`]: crate::parser::streamed::StreamedParserExt::flat_sep_by_times
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatSepByTimes<P, Q> {
    inner: P,
    sep: Q,
    count: usize,
}

impl<P, Q> FlatSepByTimes<P, Q> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, sep: Q, count: usize) -> Self {
        Self { inner, sep, count }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct FlatSepByTimesState<I, P: StreamedParser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        count: usize,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> StreamedParser<I> for FlatSepByTimes<P, Q>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = FlatSepByTimesState<I, P, Q>;

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

            if let EitherState::Right(inner) = &mut state.inner {
                match ready!(self.sep.poll_parse(input.as_mut(), cx, inner)?) {
                    (Status::Success(_, err), _) => {
                        state.inner = EitherState::new_left();
                        merge_errors(&mut state.error, err);
                    }
                    (Status::Failure(err, false), pos) => {
                        merge_errors(&mut state.error, Some(err));
                        break (
                            Status::Failure(state.error().unwrap(), false),
                            state.start()..pos.end,
                        );
                    }
                    (Status::Failure(err, true), pos) => {
                        break (Status::Failure(err, true), state.start()..pos.end);
                    }
                }
            }

            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, state.inner.left())?)
            {
                (Status::Success(Some(val), err), pos) => {
                    merge_errors(&mut state.error, err);
                    state.set_start(|| pos.start);
                    break (
                        Status::Success(Some(val), state.error()),
                        state.start()..pos.end,
                    );
                }
                (Status::Success(None, err), pos) => {
                    state.count += 1;
                    state.inner = EitherState::new_right();
                    merge_errors(&mut state.error, err);
                    state.set_start(|| pos.start);
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err));
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
