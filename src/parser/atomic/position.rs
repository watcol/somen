use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`position`].
///
/// [`position`]: crate::parser::position
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position<I: ?Sized> {
    _phantom: PhantomData<I>,
}

impl<I: ?Sized> Default for Position<I> {
    #[inline]
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<I: ?Sized> Position<I> {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<I: Positioned + ?Sized> Parser<I> for Position<I> {
    type Output = I::Locator;
    type State = ();

    #[inline]
    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        _cx: &mut Context<'_>,
        _state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        Poll::Ready(Ok(Status::Success(input.position(), None)))
    }
}
