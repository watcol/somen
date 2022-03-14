use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::merge_errors;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`reduce`].
///
/// [`reduce`]: crate::parser::streamed::StreamedParserExt::reduce
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reduce<P, F> {
    inner: P,
    f: F,
}

impl<P, F> Reduce<P, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct ReduceState<I, P: StreamedParser> {
        inner: P::State,
        acc: Option<P::Item>,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, F, I> Parser<I> for Reduce<P, F>
where
    P: StreamedParser<I>,
    F: FnMut(P::Item, P::Item) -> P::Item,
    I: Positioned + ?Sized,
{
    type Output = Option<P::Item>;
    type State = ReduceState<I, P>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        Poll::Ready(Ok(loop {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
            {
                (Status::Success(Some(val), err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                    state.acc = Some(match state.acc() {
                        Some(acc) => (self.f)(acc, val),
                        None => val,
                    });
                }
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                    break (
                        Status::Success(state.acc(), state.error()),
                        state.start()..pos.end,
                    );
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
}

/// A parser for method [`try_reduce`].
///
/// [`try_reduce`]: crate::parser::streamed::StreamedParserExt::try_reduce
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TryReduce<P, F> {
    inner: P,
    f: F,
}

impl<P, F> TryReduce<P, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, F, E, I> Parser<I> for TryReduce<P, F>
where
    P: StreamedParser<I>,
    F: FnMut(P::Item, P::Item) -> Result<P::Item, E>,
    E: Into<Expects<I::Ok>>,
    I: Positioned + ?Sized,
{
    type Output = Option<P::Item>;
    type State = ReduceState<I, P>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        Poll::Ready(Ok(loop {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
            {
                (Status::Success(Some(val), err), pos) => {
                    state.acc = Some(match state.acc() {
                        Some(acc) => match (self.f)(acc, val) {
                            Ok(res) => res,
                            Err(exp) => {
                                break (
                                    Status::Failure(
                                        Error {
                                            expects: exp.into(),
                                            position: pos.clone(),
                                        },
                                        true,
                                    ),
                                    state.start()..pos.end,
                                )
                            }
                        },
                        None => val,
                    });
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                }
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                    break (
                        Status::Success(state.acc(), state.error()),
                        state.start()..pos.end,
                    );
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
}
