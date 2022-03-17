use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, ExpectKind, Expects, PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`eof`].
///
/// [`eof`]: crate::parser::eof
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Eof<I: ?Sized>(PhantomData<I>);

impl<I: ?Sized> Default for Eof<I> {
    #[inline]
    fn default() -> Self {
        Eof(PhantomData)
    }
}

impl<I: ?Sized> Eof<I> {
    /// Creating a new instance.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<I: Positioned + ?Sized> Parser<I> for Eof<I> {
    type Output = ();
    type State = ();

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        _state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        let start = input.position();
        Poll::Ready(Ok(match ready!(input.as_mut().try_poll_next(cx)?) {
            Some(_) => Status::Failure(
                Error {
                    expects: Expects::new_neg(ExpectKind::Any),
                    position: start..input.position(),
                },
                false,
            ),
            None => Status::Success((), None),
        }))
    }
}
