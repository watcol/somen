use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use super::utils::EitherState;
use crate::error::{PolledResult, Tracker};
use crate::parser::streamed::StreamedParser;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`ahead_of`].
///
/// [`ahead_of`]: super::ParserExt::ahead_of
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AheadOf<P, Q> {
    inner: P,
    skipped: Q,
}

impl<P, Q> AheadOf<P, Q> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, skipped: Q) -> Self {
        Self { inner, skipped }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.inner, self.skipped)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AheadOfState<C, D, O> {
    inner: EitherState<C, D>,
    output: Option<O>,
}

impl<C: Default, D, O> Default for AheadOfState<C, D, O> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: EitherState::default(),
            output: None,
        }
    }
}

impl<P, Q, I> Parser<I> for AheadOf<P, Q>
where
    P: Parser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = (AheadOfState<P::State, Q::State, P::Output>, bool);

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        if let EitherState::Left(inner) = &mut state.0.inner {
            let (output, committed) =
                ready!(self.inner.poll_parse(input.as_mut(), cx, inner, tracker))?;
            state.0.output = Some(output);
            state.0.inner = EitherState::Right(Default::default());
            state.1 = committed;
        }

        self.skipped
            .poll_parse(input, cx, state.0.inner.as_mut_right(), tracker)
            .map_ok(|(_, committed)| {
                (
                    mem::take(&mut state.0.output).unwrap(),
                    state.1 || committed,
                )
            })
            .map_err(|err| err.fatal_if(state.1))
    }
}

impl<P, Q, I> StreamedParser<I> for AheadOf<P, Q>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = (EitherState<P::State, Q::State>, bool, bool);

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.0 {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, inner, tracker))?
            {
                (Some(val), committed) => {
                    state.1 |= committed;
                    return Poll::Ready(Ok((Some(val), committed)));
                }
                (None, committed) => {
                    state.1 |= committed;
                    state.2 = committed;
                    state.0 = EitherState::Right(Default::default());
                }
            }
        }

        self.skipped
            .poll_parse(input, cx, state.0.as_mut_right(), tracker)
            .map_ok(|(_, committed)| (None, state.2 || committed))
            .map_err(|err| err.fatal_if(state.1))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A parser for method [`behind`].
///
/// [`behind`]: super::ParserExt::behind
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Behind<P, Q> {
    inner: P,
    skipped: Q,
}

impl<P, Q> Behind<P, Q> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, skipped: Q) -> Self {
        Self { inner, skipped }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.inner, self.skipped)
    }
}

impl<P, Q, I> Parser<I> for Behind<P, Q>
where
    P: Parser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = (EitherState<Q::State, P::State>, bool);

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        if let EitherState::Left(inner) = &mut state.0 {
            let (_, committed) =
                ready!(self.skipped.poll_parse(input.as_mut(), cx, inner, tracker))?;
            state.0 = EitherState::Right(Default::default());
            state.1 = committed;
        }

        self.inner
            .poll_parse(input, cx, state.0.as_mut_right(), tracker)
            .map_ok(|(val, committed)| (val, state.1 || committed))
            .map_err(|err| err.fatal_if(state.1))
    }
}

impl<P, Q, I> StreamedParser<I> for Behind<P, Q>
where
    P: StreamedParser<I>,
    Q: Parser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = (EitherState<Q::State, P::State>, bool, bool);

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.0 {
            state.1 = ready!(self.skipped.poll_parse(input.as_mut(), cx, inner, tracker))?.1;
            state.2 = state.1;
            state.0 = EitherState::Right(Default::default());
        }

        self.inner
            .poll_parse_next(input, cx, state.0.as_mut_right(), tracker)
            .map_ok(|(val, committed)| {
                state.1 |= committed;
                (val, mem::take(&mut state.2) || committed)
            })
            .map_err(|err| err.fatal_if(state.1))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A parser for method [`between`].
///
/// [`between`]: super::ParserExt::between
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Between<P, L, R> {
    inner: P,
    left: L,
    right: R,
}

impl<P, L, R> Between<P, L, R> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, left: L, right: R) -> Self {
        Self { inner, left, right }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, L, R) {
        (self.inner, self.left, self.right)
    }
}

type BetweenState<L, P, R, O> = (EitherState<L, AheadOfState<P, R, O>>, bool);

impl<P, L, R, I> Parser<I> for Between<P, L, R>
where
    P: Parser<I>,
    L: Parser<I>,
    R: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = BetweenState<L::State, P::State, R::State, P::Output>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        if let EitherState::Left(inner) = &mut state.0 {
            state.1 = ready!(self.left.poll_parse(input.as_mut(), cx, inner, tracker))?.1;
            state.0 = EitherState::Right(Default::default());
        }

        let state0 = state.0.as_mut_right();
        if let EitherState::Left(inner) = &mut state0.inner {
            let (output, committed) =
                ready!(self.inner.poll_parse(input.as_mut(), cx, inner, tracker))
                    .map_err(|err| err.fatal_if(state.1))?;
            state0.output = Some(output);
            state0.inner = EitherState::Right(Default::default());
            state.1 |= committed;
        }

        self.right
            .poll_parse(input, cx, state0.inner.as_mut_right(), tracker)
            .map_ok(|(_, committed)| (mem::take(&mut state0.output).unwrap(), state.1 || committed))
            .map_err(|err| err.fatal_if(state.1))
    }
}

impl<P, L, R, I> StreamedParser<I> for Between<P, L, R>
where
    P: StreamedParser<I>,
    L: Parser<I>,
    R: Parser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = (
        EitherState<L::State, EitherState<P::State, R::State>>,
        bool,
        bool,
    );

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.0 {
            state.1 = ready!(self.left.poll_parse(input.as_mut(), cx, inner, tracker))?.1;
            state.2 = state.1;
            state.0 = EitherState::Right(Default::default());
        }

        let state0 = state.0.as_mut_right();
        if let EitherState::Left(inner) = state0 {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, inner, tracker)
                .map_err(|err| err.fatal_if(state.1))?)
            {
                (Some(val), committed) => {
                    state.1 |= committed;
                    return Poll::Ready(Ok((Some(val), mem::take(&mut state.2) || committed)));
                }
                (None, committed) => {
                    state.1 |= committed;
                    state.2 = committed;
                    *state0 = EitherState::Right(Default::default());
                }
            }
        }

        self.right
            .poll_parse(input, cx, state0.as_mut_right(), tracker)
            .map_ok(|(_, committed)| (None, state.2 || committed))
            .map_err(|err| err.fatal_if(state.1))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
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
    ) -> PolledResult<Self::Output, I> {
        self.inner
            .poll_parse(input, cx, state, tracker)
            .map_ok(|(_, committed)| ((), committed))
    }
}
