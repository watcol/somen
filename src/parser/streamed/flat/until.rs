use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::prelude::StreamedParser;
use crate::stream::Input;

/// A streamed parser generated from method [`flat_until`].
///
/// [`flat_until`]: crate::parser::streamed::StreamedParserExt::flat_until
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatUntil<P, Q> {
    inner: P,
    end: Q,
}

impl<P, Q> FlatUntil<P, Q> {
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
    pub struct FlatUntilState<I: Input, P: StreamedParser, Q: Parser> {
        inner: EitherState<Q::State, P::State>,
        #[opt]
        marker: I::Marker,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> StreamedParser<I> for FlatUntil<P, Q>
where
    P: StreamedParser<I>,
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
                }

                match ready!(self.end.poll_parse(input.as_mut(), cx, inner)?) {
                    (Status::Success(_, err), pos) => {
                        input.drop_marker(state.marker())?;
                        merge_errors(&mut state.error, err);
                        state.set_start(|| pos.start);
                        break (Status::Success(None, state.error()), state.start()..pos.end);
                    }
                    (Status::Failure(err, false), pos) if err.rewindable(&pos.start) => {
                        input.as_mut().rewind(state.marker())?;
                        state.inner = EitherState::new_right();
                        merge_errors(&mut state.error, Some(err));
                    }
                    (Status::Failure(err, false), pos) => {
                        input.drop_marker(state.marker())?;
                        merge_errors(&mut state.error, Some(err));
                        state.set_start(|| pos.start);
                        break (
                            Status::Failure(state.error().unwrap(), false),
                            state.start()..pos.end,
                        );
                    }
                    (Status::Failure(err, true), pos) => {
                        input.drop_marker(state.marker())?;
                        state.set_start(|| pos.start);
                        break (Status::Failure(err, true), state.start()..pos.end);
                    }
                }
            }

            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, state.inner.right())?)
            {
                (Status::Success(Some(val), err), pos) => {
                    merge_errors(&mut state.error, err);
                    state.set_start(|| pos.start);
                    break (
                        Status::Success(Some(val), state.error()),
                        state.start()..pos.end,
                    );
                }
                (Status::Success(None, err), pos) => {
                    state.inner = EitherState::new_left();
                    merge_errors(&mut state.error, err);
                    state.set_start(|| pos.start);
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err));
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
