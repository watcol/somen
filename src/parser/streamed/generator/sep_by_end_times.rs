use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::prelude::StreamedParser;
use crate::stream::Input;

/// A streamed parser generated from method [`sep_by_end_times`].
///
/// [`sep_by_end_times`]: crate::parser::ParserExt::sep_by_end_times
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SepByEndTimes<P, Q> {
    inner: P,
    sep: Q,
    count: usize,
}

impl<P, Q> SepByEndTimes<P, Q> {
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
    pub struct SepByEndTimesState<I: Input, P: Parser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        count: usize,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
        #[opt]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> StreamedParser<I> for SepByEndTimes<P, Q>
where
    P: Parser<I>,
    Q: Parser<I>,
    I: Input + ?Sized,
{
    type Item = P::Output;
    type State = SepByEndTimesState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        if state.count >= self.count {
            state.set_marker(|| input.as_mut().mark())?;
            return Poll::Ready(Ok(
                match ready!(self
                    .sep
                    .poll_parse(input.as_mut(), cx, state.inner.right())?)
                {
                    (Status::Success(_, err), pos) => {
                        input.drop_marker(state.marker())?;
                        (Status::Success(None, err), pos)
                    }
                    (Status::Failure(err, false), pos) if err.rewindable(&pos.start) => {
                        input.rewind(state.marker())?;
                        (
                            Status::Success(None, Some(err)),
                            pos.start.clone()..pos.start,
                        )
                    }
                    (Status::Failure(err, exclusive), pos) => {
                        (Status::Failure(err, exclusive), pos)
                    }
                },
            ));
        }

        if let EitherState::Right(inner) = &mut state.inner {
            match ready!(self.sep.poll_parse(input.as_mut(), cx, inner)?) {
                (Status::Success(_, err), pos) => {
                    state.inner = EitherState::new_left();
                    state.error = err;
                    state.start = Some(pos.start);
                }
                (Status::Failure(err, exclusive), pos) => {
                    return Poll::Ready(Ok((Status::Failure(err, exclusive), pos)))
                }
            }
        }

        Poll::Ready(Ok(
            match ready!(self
                .inner
                .poll_parse(input.as_mut(), cx, state.inner.left())?)
            {
                (Status::Success(val, err), pos) => {
                    state.count += 1;
                    state.inner = EitherState::new_right();
                    merge_errors(&mut state.error, err, &pos);
                    let start = if state.start.is_some() {
                        state.start()
                    } else {
                        pos.start
                    };
                    (Status::Success(Some(val), state.error()), start..pos.end)
                }
                (Status::Failure(err, exclusive), pos) => {
                    if exclusive {
                        state.error = Some(err);
                    } else {
                        merge_errors(&mut state.error, Some(err), &pos);
                    }

                    let start = if state.start.is_some() {
                        state.start()
                    } else {
                        pos.start
                    };
                    (
                        Status::Failure(state.error().unwrap(), exclusive),
                        start..pos.end,
                    )
                }
            },
        ))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}
