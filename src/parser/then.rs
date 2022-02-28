use core::pin::Pin;
use core::task::Context;
use futures_core::ready;

use crate::error::{Expects, ParseError, PolledResult, Tracker};
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
    type State = (ThenState<P::State, Q, Q::State>, bool);

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        if let ThenState::Left(inner) = &mut state.0 {
            let (output, committed) =
                ready!(self.inner.poll_parse(input.as_mut(), cx, inner, tracker))?;
            state.1 = committed;
            state.0 = ThenState::Right((self.f)(output), Default::default());
        }

        if let ThenState::Right(parser, inner) = &mut state.0 {
            parser
                .poll_parse(input, cx, inner, tracker)
                .map_ok(|(res, committed)| (res, state.1 | committed))
                .map_err(|err| err.fatal_if(state.1))
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
    type State = (
        SpanState<ThenState<P::State, Q, Q::State>, I::Locator>,
        bool,
    );

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        if let ThenState::Left(inner) = &mut state.0.inner {
            if state.0.start.is_none() {
                state.0.start = Some(input.position());
            }
            let (output, committed) =
                ready!(self.inner.poll_parse(input.as_mut(), cx, inner, tracker))?;
            state.1 = committed;
            state.0.inner = ThenState::Right(
                (self.f)(output).map_err(|ex| {
                    tracker.clear();
                    ParseError::Parser {
                        expects: ex.into(),
                        position: state.0.take_start()..input.position(),
                        fatal: true,
                    }
                })?,
                Default::default(),
            );
        }

        if let ThenState::Right(parser, inner) = &mut state.0.inner {
            parser
                .poll_parse(input, cx, inner, tracker)
                .map_ok(|(res, committed)| (res, state.1 | committed))
                .map_err(|err| err.fatal_if(state.1))
        } else {
            unreachable!()
        }
    }
}
