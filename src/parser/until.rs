use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, PolledResult, Tracker};
use crate::parser::Parser;
use crate::prelude::StreamedParser;
use crate::stream::Input;

use super::utils::EitherState;

/// A streamed parser generated from method [`until`].
///
/// [`until`]: super::ParserExt::until
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UntilState<C, D, M> {
    inner: EitherState<C, D>,
    queued_marker: Option<M>,
    committed: bool,
}

impl<C, D: Default, M> Default for UntilState<C, D, M> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::Right(Default::default()),
            queued_marker: None,
            committed: false,
        }
    }
}

impl<P, Q, I> StreamedParser<I> for Until<P, Q>
where
    P: Parser<I>,
    Q: Parser<I>,
    I: Input + ?Sized,
{
    type Item = P::Output;
    type State = UntilState<P::State, Q::State, I::Marker>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        if state.queued_marker.is_none() {
            state.queued_marker = Some(input.as_mut().mark()?);
        }

        if let EitherState::Right(inner) = &mut state.inner {
            match ready!(self.end.poll_parse(input.as_mut(), cx, inner, tracker)) {
                Ok((_, committed)) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    return Poll::Ready(Ok((None, committed)));
                }
                Err(ParseError::Parser {
                    expects,
                    fatal: false,
                    ..
                }) => {
                    input
                        .as_mut()
                        .rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    tracker.add(expects);
                    state.inner = EitherState::Left(Default::default());
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    return Poll::Ready(Err(err));
                }
            }
        }

        Poll::Ready(
            match ready!(self.inner.poll_parse(
                input.as_mut(),
                cx,
                state.inner.as_mut_left(),
                tracker
            )) {
                Ok((val, committed)) => {
                    state.inner = EitherState::Right(Default::default());
                    state.committed |= committed;
                    Ok((Some(val), committed))
                }
                Err(err) => Err(err.fatal_if(state.committed)),
            },
        )
    }
}
