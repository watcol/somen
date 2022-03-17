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
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, sep: Q, count: usize) -> Self {
        Self { inner, sep, count }
    }

    /// Extracts the inner parser.
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
                state.set_start(|| input.position());
                break match ready!(self
                    .sep
                    .poll_parse(input.as_mut(), cx, state.inner.right())?)
                {
                    Status::Success(_, err) => {
                        input.drop_marker(state.marker())?;
                        merge_errors(&mut state.error, err);
                        Status::Success(None, state.error())
                    }
                    Status::Failure(err, false) if err.rewindable(&state.start()) => {
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
                };
            }

            if let EitherState::Right(inner) = &mut state.inner {
                match ready!(self.sep.poll_parse(input.as_mut(), cx, inner)?) {
                    Status::Success(_, err) => {
                        state.inner = EitherState::new_left();
                        merge_errors(&mut state.error, err);
                    }
                    Status::Failure(err, false) => {
                        merge_errors(&mut state.error, Some(err));
                        break Status::Failure(state.error().unwrap(), false);
                    }
                    Status::Failure(err, true) => break Status::Failure(err, true),
                }
            }

            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, state.inner.left())?)
            {
                Status::Success(Some(val), err) => {
                    merge_errors(&mut state.error, err);
                    break Status::Success(Some(val), state.error());
                }
                Status::Success(None, err) => {
                    state.count += 1;
                    state.inner = EitherState::new_right();
                    merge_errors(&mut state.error, err);
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
