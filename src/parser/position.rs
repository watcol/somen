use core::marker::PhantomData;
use core::ops::Range;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::utils::SpanState;
use crate::error::{PolledResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`position`].
///
/// [`position`]: crate::parser::position
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Position<I: ?Sized>(PhantomData<I>);

impl<I: ?Sized> Default for Position<I> {
    #[inline]
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<I: ?Sized> Position<I> {
    /// Creating a new instance.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<I> Parser<I> for Position<I>
where
    I: Positioned + ?Sized,
{
    type Output = I::Locator;
    type State = ();

    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        _cx: &mut Context<'_>,
        _state: &mut Self::State,
        _tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        Poll::Ready(Ok((input.position(), false)))
    }
}

/// A parser for method [`with_position`].
///
/// [`with_position`]: super::ParserExt::with_position
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WithPosition<P> {
    inner: P,
}

impl<P> WithPosition<P> {
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

impl<P, I> Parser<I> for WithPosition<P>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = (P::Output, Range<I::Locator>);
    type State = SpanState<P::State, I::Locator>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        state.set_start(|| input.position());
        self.inner
            .poll_parse(input.as_mut(), cx, &mut state.inner, tracker)
            .map_ok(|(res, committed)| ((res, state.take_start()..input.position()), committed))
    }
}
