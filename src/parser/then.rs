use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Expects, ParseError, ParseResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

use super::utils::SpanState;

/// A parser for method [`then`].
///
/// [`then`]: super::ParserExt::then
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Then<P, F> {
    inner: P,
    f: F,
}

impl<P, F> Then<P, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThenState<C, Q, D> {
    Left(C),
    Right(Q, D),
}

impl<C: Default, Q, D> Default for ThenState<C, Q, D> {
    #[inline]
    fn default() -> Self {
        Self::Left(Default::default())
    }
}

impl<P, F, I, Q> Parser<I> for Then<P, F>
where
    P: Parser<I>,
    Q: Parser<I>,
    F: FnMut(P::Output) -> Q,
    I: Positioned + ?Sized,
{
    type Output = Q::Output;
    type State = ThenState<P::State, Q, Q::State>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        if let ThenState::Left(inner) = state {
            *state = ThenState::Right(
                (self.f)(ready!(self.inner.poll_parse(
                    input.as_mut(),
                    cx,
                    inner,
                    tracker
                ))?),
                Default::default(),
            );
        }

        if let ThenState::Right(parser, inner) = state {
            parser
                .poll_parse(input, cx, inner, tracker)
                .map_err(|err| err.fatal(true))
        } else {
            unreachable!()
        }
    }
}

/// A parser for method [`try_then`].
///
/// [`try_then`]: super::ParserExt::try_then
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TryThen<P, F> {
    inner: P,
    f: F,
}

impl<P, F> TryThen<P, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, F, I, Q, E> Parser<I> for TryThen<P, F>
where
    P: Parser<I>,
    Q: Parser<I>,
    E: Into<Expects<I::Ok>>,
    F: FnMut(P::Output) -> Result<Q, E>,
    I: Positioned + ?Sized,
{
    type Output = Q::Output;
    type State = SpanState<ThenState<P::State, Q, Q::State>, I::Locator>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        if let ThenState::Left(inner) = &mut state.inner {
            if state.start.is_none() {
                state.start = Some(input.position());
            }

            state.inner = ThenState::Right(
                (self.f)(ready!(self.inner.poll_parse(
                    input.as_mut(),
                    cx,
                    inner,
                    tracker
                ))?)
                .map_err(|ex| ParseError::Parser {
                    expects: ex.into(),
                    position: mem::take(&mut state.start).unwrap()..input.position(),
                    fatal: true,
                })?,
                Default::default(),
            );
        }

        if let ThenState::Right(parser, inner) = &mut state.inner {
            parser
                .poll_parse(input, cx, inner, tracker)
                .map_err(|err| err.fatal(true))
        } else {
            unreachable!()
        }
    }
}
