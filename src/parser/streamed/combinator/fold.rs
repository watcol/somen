use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`fold`].
///
/// [`fold`]: crate::parser::streamed::StreamedParserExt::fold
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fold<P, Q, F> {
    inner: P,
    init: Q,
    f: F,
}

impl<P, Q, F> Fold<P, Q, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, init: Q, f: F) -> Self {
        Self { inner, init, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.inner, self.init)
    }
}

crate::parser_state! {
    pub struct FoldState<I, P: StreamedParser, Q: Parser> {
        inner: EitherState<Q::State, P::State>,
        #[opt]
        acc: Q::Output,
        #[opt]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, F, I> Parser<I> for Fold<P, Q, F>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    F: FnMut(Q::Output, P::Item) -> Q::Output,
    I: Positioned + ?Sized,
{
    type Output = Q::Output;
    type State = FoldState<I, P, Q>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.init.poll_parse(input.as_mut(), cx, inner)?) {
                (Status::Success(acc, err), pos) => {
                    state.error = err;
                    state.start = Some(pos.start);
                    state.acc = Some(acc);
                }
                res @ (Status::Failure(_, _), _) => return Poll::Ready(Ok(res)),
            }
        }

        Poll::Ready(Ok(loop {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, state.inner.right())?)
            {
                (Status::Success(Some(val), err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.acc = Some((self.f)(state.acc(), val));
                }
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    break (
                        Status::Success(state.acc(), state.error()),
                        state.start()..pos.end,
                    );
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err), &pos);
                    break (
                        Status::Failure(state.error().unwrap(), false),
                        state.start()..pos.end,
                    );
                }
                (Status::Failure(err, true), pos) => {
                    break (Status::Failure(err, true), state.start()..pos.end);
                }
            }
        }))
    }
}

/// A parser for method [`try_fold`].
///
/// [`try_fold`]: crate::parser::streamed::StreamedParserExt::try_fold
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TryFold<P, Q, F> {
    inner: P,
    init: Q,
    f: F,
}

impl<P, Q, F> TryFold<P, Q, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, init: Q, f: F) -> Self {
        Self { inner, init, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.inner, self.init)
    }
}

impl<P, Q, F, E, I> Parser<I> for TryFold<P, Q, F>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    F: FnMut(Q::Output, P::Item) -> Result<Q::Output, E>,
    E: Into<Expects<I::Ok>>,
    I: Positioned + ?Sized,
{
    type Output = Q::Output;
    type State = FoldState<I, P, Q>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.init.poll_parse(input.as_mut(), cx, inner)?) {
                (Status::Success(acc, err), pos) => {
                    state.error = err;
                    state.start = Some(pos.start);
                    state.acc = Some(acc);
                }
                res @ (Status::Failure(_, _), _) => return Poll::Ready(Ok(res)),
            }
        }

        Poll::Ready(Ok(loop {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, state.inner.right())?)
            {
                (Status::Success(Some(val), err), pos) => match (self.f)(state.acc(), val) {
                    Ok(acc) => {
                        merge_errors(&mut state.error, err, &pos);
                        state.acc = Some(acc);
                    }
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
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    break (
                        Status::Success(state.acc(), state.error()),
                        state.start()..pos.end,
                    );
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err), &pos);
                    break (
                        Status::Failure(state.error().unwrap(), false),
                        state.start()..pos.end,
                    );
                }
                (Status::Failure(err, true), pos) => {
                    break (Status::Failure(err, true), state.start()..pos.end);
                }
            }
        }))
    }
}
