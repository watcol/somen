use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`skip`].
///
/// [`skip`]: crate::parser::ParserExt::skip
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skip<P, Q> {
    inner: P,
    skipped: Q,
}

impl<P, Q> Skip<P, Q> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, skipped: Q) -> Self {
        Self { inner, skipped }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.inner, self.skipped)
    }
}

crate::parser_state! {
    pub struct SkipState<I, P: Parser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        #[opt]
        output: P::Output,
        #[opt]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> Parser<I> for Skip<P, Q>
where
    P: Parser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = SkipState<I, P, Q>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.inner.poll_parse(input.as_mut(), cx, inner)?) {
                (Status::Success(val, err), pos) => {
                    state.output = Some(val);
                    state.inner = EitherState::new_right();
                    state.start = Some(pos.start);
                    state.error = err;
                }
                failure @ (Status::Failure(_, _), _) => return Poll::Ready(Ok(failure)),
            }
        }

        self.skipped
            .poll_parse(input, cx, state.inner.right())
            .map_ok(|status| match status {
                (Status::Success(_, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    (
                        Status::Success(state.output(), state.error()),
                        state.start()..pos.end,
                    )
                }
                (Status::Failure(err, exclusive), pos) => {
                    if exclusive {
                        state.error = Some(err);
                    } else {
                        merge_errors(&mut state.error, Some(err), &pos);
                    }
                    (
                        Status::Failure(state.error().unwrap(), exclusive),
                        state.start()..pos.end,
                    )
                }
            })
    }
}

crate::parser_state! {
    pub struct SkipStreamedState<I, P: StreamedParser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        #[opt]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> StreamedParser<I> for Skip<P, Q>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = SkipStreamedState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.inner.poll_parse_next(input.as_mut(), cx, inner))? {
                some @ (Status::Success(Some(_), _), _) => return Poll::Ready(Ok(some)),
                (Status::Success(None, err), pos) => {
                    state.inner = EitherState::new_right();
                    state.start = Some(pos.start);
                    state.error = err;
                }
                failure @ (Status::Failure(_, _), _) => return Poll::Ready(Ok(failure)),
            }
        }

        self.skipped
            .poll_parse(input, cx, state.inner.right())
            .map_ok(|status| match status {
                (Status::Success(_, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    (Status::Success(None, state.error()), state.start()..pos.end)
                }
                (Status::Failure(err, exclusive), pos) => {
                    if exclusive {
                        state.error = Some(err);
                    } else {
                        merge_errors(&mut state.error, Some(err), &pos);
                    }
                    (
                        Status::Failure(state.error().unwrap(), exclusive),
                        state.start()..pos.end,
                    )
                }
            })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
