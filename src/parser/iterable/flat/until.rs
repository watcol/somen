use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::prelude::IterableParser;
use crate::stream::Input;

/// A iterable parser generated from method [`flat_until`].
///
/// [`flat_until`]: crate::parser::iterable::IterableParserExt::flat_until
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatUntil<P, Q> {
    inner: P,
    end: Q,
}

impl<P, Q> FlatUntil<P, Q> {
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
    pub struct FlatUntilState<I: Input, P: IterableParser, Q: Parser> {
        inner: EitherState<Q::State, P::State>,
        #[opt]
        marker: I::Marker,
        #[opt]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> IterableParser<I> for FlatUntil<P, Q>
where
    P: IterableParser<I>,
    Q: Parser<I>,
    I: Input + ?Sized,
{
    type Item = P::Item;
    type State = FlatUntilState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        Poll::Ready(Ok(loop {
            if let EitherState::Left(inner) = &mut state.inner {
                if state.marker.is_none() {
                    state.marker = Some(input.as_mut().mark()?);
                    state.start = Some(input.position());
                }

                match ready!(self.end.poll_parse(input.as_mut(), cx, inner)?) {
                    Status::Success(_, err) => {
                        input.drop_marker(state.marker())?;
                        state.start = None;
                        merge_errors(&mut state.error, err);
                        break Status::Success(None, state.error());
                    }
                    Status::Failure(err, false) if err.rewindable(&state.start()) => {
                        input.as_mut().rewind(state.marker())?;
                        state.inner = EitherState::new_right();
                        merge_errors(&mut state.error, Some(err));
                    }
                    Status::Failure(err, false) => {
                        input.drop_marker(state.marker())?;
                        merge_errors(&mut state.error, Some(err));
                        break Status::Failure(state.error().unwrap(), false);
                    }
                    Status::Failure(err, true) => {
                        input.drop_marker(state.marker())?;
                        break Status::Failure(err, true);
                    }
                }
            }

            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, state.inner.right())?)
            {
                Status::Success(Some(val), err) => {
                    merge_errors(&mut state.error, err);
                    break Status::Success(Some(val), state.error());
                }
                Status::Success(None, err) => {
                    state.inner = EitherState::new_left();
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
}
