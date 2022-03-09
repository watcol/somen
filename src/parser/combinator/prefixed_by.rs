use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`prefixed_by`].
///
/// [`prefixed_by`]: crate::parser::ParserExt::prefixed_by
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefixedBy<P, Q> {
    inner: P,
    prefix: Q,
}

impl<P, Q> PrefixedBy<P, Q> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, prefix: Q) -> Self {
        Self { inner, prefix }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.inner, self.prefix)
    }
}

crate::parser_state! {
    pub struct PrefixedByState<I, P: Parser, Q: Parser> {
        inner: EitherState<Q::State, P::State>,
        #[opt]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> Parser<I> for PrefixedBy<P, Q>
where
    P: Parser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = PrefixedByState<I, P, Q>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.prefix.poll_parse(input.as_mut(), cx, inner)?) {
                (Status::Success(_, err), pos) => {
                    state.inner = EitherState::new_right();
                    state.start = Some(pos.start);
                    state.error = err;
                }
                (Status::Failure(err, exclusive), pos) => {
                    return Poll::Ready(Ok((Status::Failure(err, exclusive), pos)))
                }
            }
        }

        self.inner
            .poll_parse(input, cx, state.inner.right())
            .map_ok(|status| match status {
                (Status::Success(val, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    (Status::Success(val, state.error()), state.start()..pos.end)
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
    pub struct PrefixedByStreamedState<I, P: StreamedParser, Q: Parser> {
        inner: EitherState<Q::State, P::State>,
        #[opt]
        start: I::Locator,
        succeeded: bool,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> StreamedParser<I> for PrefixedBy<P, Q>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = PrefixedByStreamedState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.prefix.poll_parse(input.as_mut(), cx, inner)?) {
                (Status::Success(_, err), pos) => {
                    state.inner = EitherState::new_right();
                    state.start = Some(pos.start);
                    state.error = err;
                }
                (Status::Failure(err, exclusive), pos) => {
                    return Poll::Ready(Ok((Status::Failure(err, exclusive), pos)))
                }
            }
        }

        if state.succeeded {
            return self.inner.poll_parse_next(input, cx, state.inner.right());
        }

        self.inner
            .poll_parse_next(input, cx, state.inner.right())
            .map_ok(|status| match status {
                (Status::Success(val, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.succeeded = true;
                    (Status::Success(val, state.error()), state.start()..pos.end)
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
