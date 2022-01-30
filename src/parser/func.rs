use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{ParseError, ParseResult};
use crate::stream::Positioned;

use super::Parser;

/// A parser for function [`function`].
///
/// [`function`]: super::function
#[derive(Debug)]
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

impl<F, I, O, E, C> Parser<I> for Function<F, I, C>
where
    F: FnMut(
        Pin<&mut I>,
        &mut Context<'_>,
        &mut C,
    ) -> Poll<Result<O, ParseError<E, I::Error, I::Locator>>>,
    I: Positioned + ?Sized,
    C: Default,
{
    type Output = O;
    type Error = E;
    type State = C;

    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        (self.f)(input, cx, state)
    }
}
