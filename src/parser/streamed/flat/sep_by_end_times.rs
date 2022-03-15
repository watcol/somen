use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::stream::Input;

/// A streamed parser generated from method [`flat_sep_by_end_times`].
///
/// [`flat_sep_by_end_times`]: crate::parser::streamed::StreamedParserExt::flat_sep_by_end_times
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatSepByEndTimes<P, Q> {
    inner: P,
    sep: Q,
    count: usize,
}

impl<P, Q> FlatSepByEndTimes<P, Q> {
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
    pub struct FlatSepByEndTimesState<I: Input, P: StreamedParser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        count: usize,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> StreamedParser<I> for FlatSepByEndTimes<P, Q>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    I: Input + ?Sized,
{
    type Item = P::Item;
    type State = FlatSepByEndTimesState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        Poll::Ready(Ok(loop {
            if state.count >= self.count {
                state.set_marker(|| input.as_mut().mark())?;
                break match ready!(self
                    .sep
                    .poll_parse(input.as_mut(), cx, state.inner.right())?)
                {
                    (Status::Success(_, err), pos) => {
                        input.drop_marker(state.marker())?;
                        merge_errors(&mut state.error, err);
                        (Status::Success(None, state.error()), state.start()..pos.end)
                    }
                    (Status::Failure(err, false), pos) if err.rewindable(&pos.start) => {
                        input.rewind(state.marker())?;
                        merge_errors(&mut state.error, Some(err));
                        (
                            Status::Success(None, state.error()),
                            state.start()..pos.start,
                        )
                    }
                    (Status::Failure(err, false), pos) => {
                        input.drop_marker(state.marker())?;
                        merge_errors(&mut state.error, Some(err));
                        (
                            Status::Failure(state.error().unwrap(), false),
                            state.start()..pos.end,
                        )
                    }
                    (Status::Failure(err, true), pos) => {
                        input.drop_marker(state.marker())?;
                        (Status::Failure(err, true), state.start()..pos.end)
                    }
                };
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
