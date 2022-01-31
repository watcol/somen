use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Expect, Expects, ParseError, ParseResult};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`any`].
///
/// [`any`]: crate::parser::any
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Any<I: ?Sized>(PhantomData<I>);

impl<I: ?Sized> Default for Any<I> {
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
    ) -> Poll<ParseResult<Self::Output, I>> {
        let start = input.position();
        let parsed = ready!(input.as_mut().try_poll_next(cx)?);
        Poll::Ready(match parsed {
            Some(i) => Ok(i),
            None => Err(ParseError::Parser(
                Expects::new(Expect::Static("a token")),
                start..input.position(),
            )),
        })
    }
}
