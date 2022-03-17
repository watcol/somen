use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{PolledResult, Status};
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
    /// Creates a new instance.
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

    #[inline]
    fn poll_parse(
        &mut self,
        _input: Pin<&mut I>,
        _cx: &mut Context<'_>,
        _state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        Poll::Ready(Ok(Status::Success(self.value.clone(), None)))
    }
}

/// A parser for function [`value_fn`].
///
/// [`value_fn`]: crate::parser::value_fn
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValueFn<I: ?Sized, F> {
    f: F,
    _phantom: PhantomData<I>,
}

impl<I: ?Sized, F> ValueFn<I, F> {
    /// Creates a new instance.
    #[inline]
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<I: Positioned + ?Sized, F: FnMut() -> T, T> Parser<I> for ValueFn<I, F> {
    type Output = T;
    type State = ();

    #[inline]
    fn poll_parse(
        &mut self,
        _input: Pin<&mut I>,
        _cx: &mut Context<'_>,
        _state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        Poll::Ready(Ok(Status::Success((self.f)(), None)))
    }
}
