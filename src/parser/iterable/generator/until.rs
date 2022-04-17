use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::prelude::IterableParser;
use crate::stream::Input;

/// A iterable parser generated from method [`until`].
///
/// [`until`]: crate::parser::ParserExt::until
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Until<P, Q> {
    inner: P,
    end: Q,
}

impl<P, Q> Until<P, Q> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, end: Q) -> Self {
        Self { inner, end }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.inner, self.end)
    }
}

crate::parser_state! {
    pub struct UntilState<I: Input, P: Parser, Q: Parser> {
        inner: EitherState<Q::State, P::State>,
        #[opt]
        marker: I::Marker,
        #[opt]
        start: I::Locator,
        error: Option<Error<I::Locator>>,
    }
}

impl<P, Q, I> IterableParser<I> for Until<P, Q>
where
    P: Parser<I>,
    Q: Parser<I>,
    I: Input + ?Sized,
{
    type Item = P::Output;
    type State = UntilState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            if state.marker.is_none() {
                state.marker = Some(input.as_mut().mark()?);
                state.start = Some(input.position());
            }

            match ready!(self.end.poll_parse(input.as_mut(), cx, inner)?) {
                Status::Success(_, err) => {
                    input.drop_marker(state.marker())?;
                    state.start = None;
                    return Poll::Ready(Ok(Status::Success(None, err)));
                }
                Status::Failure(err, false) if err.rewindable(&state.start()) => {
                    input.as_mut().rewind(state.marker())?;
                    state.inner = EitherState::new_right();
                    state.error = Some(err);
                }
                Status::Failure(err, exclusive) => {
                    input.drop_marker(state.marker())?;
                    return Poll::Ready(Ok(Status::Failure(err, exclusive)));
                }
            }
        }

        Poll::Ready(Ok(
            match ready!(self
                .inner
                .poll_parse(input.as_mut(), cx, state.inner.right())?)
            {
                Status::Success(val, err) => {
                    state.inner = EitherState::new_left();
                    merge_errors(&mut state.error, err);
                    Status::Success(Some(val), state.error())
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    Status::Failure(state.error().unwrap(), false)
                }
                Status::Failure(err, true) => Status::Failure(err, true),
            },
        ))
    }
}
