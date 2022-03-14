use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::prelude::StreamedParser;
use crate::stream::Input;

/// A streamed parser generated from method [`until`].
///
/// [`until`]: crate::parser::ParserExt::until
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Until<P, Q> {
    inner: P,
    end: Q,
}

impl<P, Q> Until<P, Q> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, end: Q) -> Self {
        Self { inner, end }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.inner, self.end)
    }
}

crate::parser_state! {
    pub struct UntilState<I: Input, P: Parser, Q: Parser> {
        inner: EitherState<Q::State, P::State>,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> StreamedParser<I> for Until<P, Q>
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
        state.set_marker(|| input.as_mut().mark())?;

        if let EitherState::Left(inner) = &mut state.inner {
            match ready!(self.end.poll_parse(input.as_mut(), cx, inner)?) {
                (Status::Success(_, err), pos) => {
                    input.drop_marker(state.marker())?;
                    return Poll::Ready(Ok((Status::Success(None, err), pos)));
                }
                (Status::Failure(err, false), pos) if err.rewindable(&pos.start) => {
                    input.as_mut().rewind(state.marker())?;
                    state.inner = EitherState::new_right();
                    state.error = Some(err);
                }
                (Status::Failure(err, exclusive), pos) => {
                    return Poll::Ready(Ok((Status::Failure(err, exclusive), pos)))
                }
            }
        }

        Poll::Ready(Ok(
            match ready!(self
                .inner
                .poll_parse(input.as_mut(), cx, state.inner.right())?)
            {
                (Status::Success(val, err), pos) => {
                    state.inner = EitherState::new_left();
                    merge_errors(&mut state.error, err, &pos);
                    (Status::Success(Some(val), state.error()), pos)
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err), &pos);
                    (Status::Failure(state.error().unwrap(), false), pos)
                }
                (Status::Failure(err, true), pos) => (Status::Failure(err, true), pos),
            },
        ))
    }
}
