use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`prefix`].
///
/// [`prefix`]: crate::parser::ParserExt::prefix
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prefix<P, Q> {
    prefix: P,
    inner: Q,
}

impl<P, Q> Prefix<P, Q> {
    /// Creating a new instance.
    #[inline]
    pub fn new(prefix: P, inner: Q) -> Self {
        Self {
            prefix,
            inner,
        }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.prefix, self.inner)
    }
}

crate::parser_state! {
    pub struct PrefixedByState<I, P: Parser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        #[opt]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> Parser<I> for Prefix<P, Q>
where
    P: Parser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = Q::Output;
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
                    merge_errors(&mut state.error, err);
                    (Status::Success(val, state.error()), state.start()..pos.end)
                }
                (Status::Failure(err, exclusive), pos) => {
                    if exclusive {
                        state.error = Some(err);
                    } else {
                        merge_errors(&mut state.error, Some(err));
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
    pub struct PrefixedByStreamedState<I, P: Parser, Q: StreamedParser> {
        inner: EitherState<P::State, Q::State>,
        #[opt]
        start: I::Locator,
        succeeded: bool,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> StreamedParser<I> for Prefix<P, Q>
where
    P: Parser<I>,
    Q: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Item = Q::Item;
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
                    merge_errors(&mut state.error, err);
                    state.succeeded = true;
                    (Status::Success(val, state.error()), state.start()..pos.end)
                }
                (Status::Failure(err, exclusive), pos) => {
                    if exclusive {
                        state.error = Some(err);
                    } else {
                        merge_errors(&mut state.error, Some(err));
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
