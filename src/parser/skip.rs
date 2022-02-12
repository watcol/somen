use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use super::utils::EitherState;
use crate::error::{ParseResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`skip`].
///
/// [`skip`]: super::ParserExt::skip
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skip<P, Q> {
    left: P,
    right: Q,
}

impl<P, Q> Skip<P, Q> {
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
pub struct SkipState<C, D, O> {
    inner: EitherState<C, D>,
    output: Option<O>,
}

impl<C: Default, D, O> Default for SkipState<C, D, O> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::default(),
            output: None,
        }
    }
}

impl<P, Q, I> Parser<I> for Skip<P, Q>
where
    P: Parser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = SkipState<P::State, Q::State, P::Output>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        if let EitherState::Left(ref mut inner) = state.inner {
            state.output = Some(ready!(self.left.poll_parse(
                input.as_mut(),
                cx,
                inner,
                tracker
            ))?);
            state.inner = EitherState::Right(Default::default());
        }

        self.right
            .poll_parse(input, cx, state.inner.as_mut_right(), tracker)
            .map_ok(|_| mem::take(&mut state.output).unwrap())
            .map_err(|err| err.fatal(true))
    }
}

/// A parser for method [`skip_to`].
///
/// [`skip_to`]: super::ParserExt::skip_to
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkipTo<P, Q> {
    left: P,
    right: Q,
}

impl<P, Q> SkipTo<P, Q> {
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

impl<P, Q, I> Parser<I> for SkipTo<P, Q>
where
    P: Parser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = Q::Output;
    type State = EitherState<P::State, Q::State>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        if let EitherState::Left(inner) = state {
            ready!(self.left.poll_parse(input.as_mut(), cx, inner, tracker))?;
            *state = EitherState::Right(Default::default());
        }

        self.right
            .poll_parse(input, cx, state.as_mut_right(), tracker)
            .map_err(|err| err.fatal(true))
    }
}

/// A parser for method [`discard`].
///
/// [`discard`]: super::ParserExt::discard
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Discard<P> {
    inner: P,
}

impl<P> Discard<P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P) -> Self {
        Self { inner }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, I> Parser<I> for Discard<P>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = ();
    type State = P::State;

    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<<I>::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        self.inner
            .poll_parse(input, cx, state, tracker)
            .map_ok(|_| ())
    }
}
