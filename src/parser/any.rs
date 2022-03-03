use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, Expect, Expects, PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`any`].
///
/// [`any`]: crate::parser::any
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Any<I: ?Sized>(PhantomData<I>);

impl<I: ?Sized> Default for Any<I> {
    #[inline]
    fn default() -> Self {
        Any(PhantomData)
    }
}

impl<I: ?Sized> Any<I> {
    /// Creating a new instance.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<I: Positioned + ?Sized> Parser<I> for Any<I> {
    type Output = I::Ok;
    type State = ();

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        _state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        let start = input.position();
        Poll::Ready(Ok(match ready!(input.as_mut().try_poll_next(cx)?) {
            Some(i) => (Status::Success(i, None), start..input.position()),
            None => (
                Status::Fail(
                    Error {
                        expects: Expects::new(Expect::Static("a token")),
                        position: start.clone()..start.clone(),
                    },
                    false,
                ),
                start.clone()..start,
            ),
        }))
    }
}
