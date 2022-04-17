use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::iterable::IterableParser;
use crate::parser::utils::merge_errors;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`reduce`].
///
/// [`reduce`]: crate::parser::iterable::IterableParserExt::reduce
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reduce<P, F> {
    inner: P,
    f: F,
}

impl<P, F> Reduce<P, F> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct ReduceState<I, P: IterableParser> {
        inner: P::State,
        acc: Option<P::Item>,
        error: Option<Error<I::Locator>>,
    }
}

impl<P, F, I> Parser<I> for Reduce<P, F>
where
    P: IterableParser<I>,
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
                Status::Success(Some(val), err) => {
                    merge_errors(&mut state.error, err);
                    state.acc = Some(match state.acc() {
                        Some(acc) => (self.f)(acc, val),
                        None => val,
                    });
                }
                Status::Success(None, err) => {
                    merge_errors(&mut state.error, err);
                    break Status::Success(state.acc(), state.error());
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    break Status::Failure(state.error().unwrap(), false);
                }
                Status::Failure(err, true) => {
                    break Status::Failure(err, true);
                }
            }
        }))
    }
}

/// A parser for method [`try_reduce`].
///
/// [`try_reduce`]: crate::parser::iterable::IterableParserExt::try_reduce
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TryReduce<P, F> {
    inner: P,
    f: F,
}

impl<P, F> TryReduce<P, F> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct TryReduceState<I, P: IterableParser> {
        inner: P::State,
        acc: Option<P::Item>,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Locator>>,
    }
}

impl<P, F, E, I> Parser<I> for TryReduce<P, F>
where
    P: IterableParser<I>,
    F: FnMut(P::Item, P::Item) -> Result<P::Item, E>,
    E: Into<Expects>,
    I: Positioned + ?Sized,
{
    type Output = Option<P::Item>;
    type State = TryReduceState<I, P>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        Poll::Ready(Ok(loop {
            state.set_start(|| input.position());
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
            {
                Status::Success(Some(val), err) => {
                    merge_errors(&mut state.error, err);
                    state.acc = Some(match state.acc() {
                        Some(acc) => match (self.f)(acc, val) {
                            Ok(res) => {
                                state.start = None;
                                res
                            }
                            Err(exp) => {
                                break Status::Failure(
                                    Error {
                                        expects: exp.into(),
                                        position: state.start()..input.position(),
                                    },
                                    true,
                                )
                            }
                        },
                        None => {
                            state.start = None;
                            val
                        }
                    });
                }
                Status::Success(None, err) => {
                    merge_errors(&mut state.error, err);
                    state.start = None;
                    break Status::Success(state.acc(), state.error());
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    state.start = None;
                    break Status::Failure(state.error().unwrap(), false);
                }
                Status::Failure(err, true) => {
                    state.start = None;
                    break Status::Failure(err, true);
                }
            }
        }))
    }
}
