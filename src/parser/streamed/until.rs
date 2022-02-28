use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, PolledResult, Tracker};
use crate::parser::utils::EitherState;
use crate::parser::Parser;
use crate::prelude::StreamedParser;
use crate::stream::Input;

/// A parser for method [`flat_until`].
///
/// [`flat_until`]: super::StreamedParserExt::flat_until
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatUntilState<C, D, M> {
    inner: EitherState<C, D>,
    queued_marker: Option<M>,
    committed: bool,
    prev_committed: bool,
    stream_committed: bool,
}

impl<C, D: Default, M> Default for FlatUntilState<C, D, M> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::Right(Default::default()),
            queued_marker: None,
            committed: false,
            prev_committed: false,
            stream_committed: false,
        }
    }
}

impl<P, Q, I> StreamedParser<I> for FlatUntil<P, Q>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    I: Input + ?Sized,
{
    type Item = P::Item;
    type State = FlatUntilState<P::State, Q::State, I::Marker>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        loop {
            if state.queued_marker.is_none() {
                state.queued_marker = Some(input.as_mut().mark()?);
            }

            if let EitherState::Right(inner) = &mut state.inner {
                match ready!(self.end.poll_parse(input.as_mut(), cx, inner, tracker)) {
                    Ok((_, committed)) => {
                        input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                        break Poll::Ready(Ok((None, state.prev_committed || committed)));
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
                        break Poll::Ready(Err(err));
                    }
                }
            }

            match ready!(self.inner.poll_parse_next(
                input.as_mut(),
                cx,
                state.inner.as_mut_left(),
                tracker
            )) {
                Ok((Some(val), committed)) => {
                    state.stream_committed |= committed;
                    break Poll::Ready(Ok((
                        Some(val),
                        mem::take(&mut state.prev_committed) || committed,
                    )));
                }
                Ok((None, committed)) => {
                    state.committed |= mem::take(&mut state.stream_committed) || committed;
                    state.prev_committed |= committed;
                    state.inner = EitherState::Right(Default::default());
                }
                Err(err) => break Poll::Ready(Err(err.fatal_if(state.committed))),
            }
        }
    }
}
