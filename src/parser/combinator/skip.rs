use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::iterable::IterableParser;
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
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, skipped: Q) -> Self {
        Self { inner, skipped }
    }

    /// Extracts the inner parser.
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
        error: Option<Error<I::Locator>>,
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
                Status::Success(val, err) => {
                    state.output = Some(val);
                    state.inner = EitherState::new_right();
                    state.error = err;
                }
                failure => return Poll::Ready(Ok(failure)),
            }
        }

        self.skipped
            .poll_parse(input, cx, state.inner.right())
            .map_ok(|status| match status {
                Status::Success(_, err) => {
                    merge_errors(&mut state.error, err);
                    Status::Success(state.output(), state.error())
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    Status::Failure(state.error().unwrap(), false)
                }
                Status::Failure(err, true) => Status::Failure(err, true),
            })
    }
}

crate::parser_state! {
    pub struct SkipIterableState<I, P: IterableParser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        error: Option<Error<I::Locator>>,
    }
}

impl<P, Q, I> IterableParser<I> for Skip<P, Q>
where
    P: IterableParser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = SkipIterableState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.inner.poll_parse_next(input.as_mut(), cx, inner))? {
                Status::Success(None, err) => {
                    state.inner = EitherState::new_right();
                    state.error = err;
                }
                res => return Poll::Ready(Ok(res)),
            }
        }

        self.skipped
            .poll_parse(input, cx, state.inner.right())
            .map_ok(|status| match status {
                Status::Success(_, err) => {
                    merge_errors(&mut state.error, err);
                    Status::Success(None, state.error())
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    Status::Failure(state.error().unwrap(), false)
                }
                Status::Failure(err, true) => Status::Failure(err, true),
            })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
