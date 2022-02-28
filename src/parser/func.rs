use core::marker::PhantomData;
use core::pin::Pin;
use core::task::Context;

use super::streamed::StreamedParser;
use super::Parser;
use crate::error::{PolledResult, Tracker};
use crate::stream::Positioned;

/// A parser for function [`function`].
///
/// [`function`]: super::function
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function<F, I: ?Sized, C> {
    f: F,
    _phantom: PhantomData<fn(I, C)>,
}

impl<F, I: ?Sized, C> Function<F, I, C> {
    /// Creating a new instance.
    #[inline]
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<F, I, O, C> Parser<I> for Function<F, I, C>
where
    F: FnMut(Pin<&mut I>, &mut Context<'_>, &mut C, &mut Tracker<I::Ok>) -> PolledResult<O, I>,
    I: Positioned + ?Sized,
    C: Default,
{
    type Output = O;
    type State = C;

    #[inline]
    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        (self.f)(input, cx, state, tracker)
    }
}

impl<F, I, T, C> StreamedParser<I> for Function<F, I, C>
where
    F: FnMut(
        Pin<&mut I>,
        &mut Context<'_>,
        &mut C,
        &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<T>, I>,
    I: Positioned + ?Sized,
    C: Default,
{
    type Item = T;
    type State = C;

    #[inline]
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Option<Self::Item>, I> {
        (self.f)(input, cx, state, tracker)
    }
}
