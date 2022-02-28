use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Expect, Expects, ParseError, PolledResult, Tracker};
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
    I::Ok: Clone + PartialEq,
{
    type Output = I::Ok;
    type State = ();

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        _state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        let start = input.position();
        Poll::Ready(match ready!(input.as_mut().try_poll_next(cx)?) {
            Some(i) if i == self.token => {
                tracker.clear();
                Ok((i, true))
            }
            _ => Err(ParseError::Parser {
                expects: Expects::new(Expect::Token(self.token.clone())),
                position: start..input.position(),
                fatal: false,
            }),
        })
    }
}

/// A parser for function [`not`].
///
/// [`not`]: crate::parser::not
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Not<I: ?Sized, T> {
    token: T,
    _phantom: PhantomData<I>,
}

impl<I: ?Sized, T> Not<I, T> {
    /// Creating a new instance.
    #[inline]
    pub fn new(token: T) -> Self {
        Self {
            token,
            _phantom: PhantomData,
        }
    }
}

impl<I> Parser<I> for Not<I, I::Ok>
where
    I: Positioned + ?Sized,
    I::Ok: Clone + PartialEq,
{
    type Output = I::Ok;
    type State = ();

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        _state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        let start = input.position();
        Poll::Ready(match ready!(input.as_mut().try_poll_next(cx)?) {
            Some(i) if i != self.token => {
                tracker.clear();
                Ok((i, true))
            }
            _ => Err(ParseError::Parser {
                expects: Expects::new(Expect::Token(self.token.clone())),
                position: start..input.position(),
                fatal: false,
            }),
        })
    }
}
