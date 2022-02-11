use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{ParseResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`value`].
///
/// [`value`]: crate::parser::value
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Value<I: ?Sized, T> {
    value: T,
    _phantom: PhantomData<I>,
}

impl<I: ?Sized, T> Value<I, T> {
    /// Creating a new instance.
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }
}

impl<I: Positioned + ?Sized, T: Clone> Parser<I> for Value<I, T> {
    type Output = T;
    type State = ();

    fn poll_parse(
        &mut self,
        _input: Pin<&mut I>,
        _cx: &mut Context<'_>,
        _state: &mut Self::State,
        _tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        Poll::Ready(Ok(self.value.clone()))
    }
}
