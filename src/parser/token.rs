use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Expect, Expects, ParseError, ParseResult};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`token`].
///
/// [`token`]: crate::parser::token
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token<I: ?Sized, T> {
    token: T,
    _phantom: PhantomData<I>,
}

impl<I: ?Sized, T> Token<I, T> {
    /// Creating a new instance.
    #[inline]
    pub fn new(token: T) -> Self {
        Self {
            token,
            _phantom: PhantomData,
        }
    }
}

impl<I> Parser<I> for Token<I, I::Ok>
where
    I: Positioned + ?Sized,
    I::Ok: Clone + Eq,
{
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
            Some(i) if i == self.token => Ok(i),
            _ => Err(ParseError::Parser {
                expects: Expects::new(Expect::Token(self.token.clone())),
                position: start..input.position(),
                fatal: false,
            }),
        })
    }
}
