use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use super::utils::EitherState;
use crate::error::{ParseError, PolledResult, Tracker};
use crate::parser::streamed::StreamedParser;
use crate::parser::Parser;
use crate::stream::Input;

/// A parser for method [`or`].
///
/// [`or`]: super::ParserExt::or
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Or<P, Q> {
    left: P,
    right: Q,
}

impl<P, Q> Or<P, Q> {
    /// Creating a new instance.
    #[inline]
    pub fn new(left: P, right: Q) -> Self {
        Self { left, right }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.left, self.right)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrState<C, D, M> {
    inner: EitherState<C, D>,
    queued_marker: Option<M>,
}

impl<C: Default, D, M> Default for OrState<C, D, M> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::default(),
            queued_marker: None,
        }
    }
}

impl<P, Q, I> Parser<I> for Or<P, Q>
where
    P: Parser<I>,
    Q: Parser<I, Output = P::Output>,
    I: Input + ?Sized,
{
    type Output = P::Output;
    type State = OrState<P::State, Q::State, I::Marker>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            if state.queued_marker.is_none() {
                state.queued_marker = Some(input.as_mut().mark()?);
            }

            match ready!(self.left.poll_parse(input.as_mut(), cx, inner, tracker)) {
                Ok(i) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    return Poll::Ready(Ok(i));
                }
                Err(ParseError::Parser {
                    fatal: false,
                    expects,
                    ..
                }) => {
                    input
                        .as_mut()
                        .rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    tracker.add(expects);
                    state.inner = EitherState::Right(Default::default());
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    return Poll::Ready(Err(err));
                }
            }
        }

        self.right
            .poll_parse(input, cx, state.inner.as_mut_right(), tracker)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrStreamedState<C, D, M> {
    inner: EitherState<C, D>,
    queued_marker: Option<M>,
    succeeded: bool,
}

impl<C: Default, D, M> Default for OrStreamedState<C, D, M> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::default(),
            queued_marker: None,
            succeeded: false,
        }
    }
}

impl<P, Q, I> StreamedParser<I> for Or<P, Q>
where
    P: StreamedParser<I>,
    Q: StreamedParser<I, Item = P::Item>,
    I: Input + ?Sized,
{
    type Item = P::Item;
    type State = OrStreamedState<P::State, Q::State, I::Marker>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            if state.succeeded {
                return self.left.poll_parse_next(input, cx, inner, tracker);
            }

            if state.queued_marker.is_none() {
                state.queued_marker = Some(input.as_mut().mark()?);
            }

            match ready!(self
                .left
                .poll_parse_next(input.as_mut(), cx, inner, tracker))
            {
                Ok(val) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    state.succeeded = true;
                    return Poll::Ready(Ok(val));
                }
                Err(ParseError::Parser {
                    fatal: false,
                    expects,
                    ..
                }) => {
                    input
                        .as_mut()
                        .rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    tracker.add(expects);
                    state.inner = EitherState::Right(Default::default());
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    return Poll::Ready(Err(err));
                }
            }
        }

        self.right
            .poll_parse_next(input, cx, state.inner.as_mut_right(), tracker)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lmin, lmax) = self.left.size_hint();
        let (rmin, rmax) = self.right.size_hint();
        (
            core::cmp::min(lmin, rmin),
            lmax.zip(rmax).map(|(a, b)| core::cmp::max(a, b)),
        )
    }
}
