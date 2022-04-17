#[cfg(feature = "alloc")]
use alloc::{format, string::ToString};
#[cfg(feature = "alloc")]
use core::fmt::Display;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, Expects, PolledResult, Status};
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
    /// Creates a new instance.
    #[inline]
    pub fn new(token: T) -> Self {
        Self {
            token,
            _phantom: PhantomData,
        }
    }
}

#[cfg(feature = "alloc")]
impl<I> Parser<I> for Token<I, I::Ok>
where
    I: Positioned + ?Sized,
    I::Ok: PartialEq + Display,
{
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
            Some(i) if i == self.token => Status::Success(i, None),
            _ => Status::Failure(
                Error {
                    expects: Expects::from(self.token.to_string()),
                    position: start..input.position(),
                },
                false,
            ),
        }))
    }
}

#[cfg(not(feature = "alloc"))]
impl<I> Parser<I> for Token<I, I::Ok>
where
    I: Positioned + ?Sized,
    I::Ok: PartialEq,
{
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
            Some(i) if i == self.token => Status::Success(i, None),
            _ => Status::Failure(
                Error {
                    expects: Expects::from("<token>"),
                    position: start..input.position(),
                },
                false,
            ),
        }))
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
    /// Creates a new instance.
    #[inline]
    pub fn new(token: T) -> Self {
        Self {
            token,
            _phantom: PhantomData,
        }
    }
}

#[cfg(feature = "alloc")]
impl<I> Parser<I> for Not<I, I::Ok>
where
    I: Positioned + ?Sized,
    I::Ok: PartialEq + Display,
{
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
            Some(i) if i != self.token => Status::Success(i, None),
            _ => Status::Failure(
                Error {
                    expects: Expects::from(format!("not {}", self.token)),
                    position: start..input.position(),
                },
                false,
            ),
        }))
    }
}

#[cfg(not(feature = "alloc"))]
impl<I> Parser<I> for Not<I, I::Ok>
where
    I: Positioned + ?Sized,
    I::Ok: PartialEq,
{
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
            Some(i) if i != self.token => Status::Success(i, None),
            _ => Status::Failure(
                Error {
                    expects: Expects::from("<not token>"),
                    position: start..input.position(),
                },
                false,
            ),
        }))
    }
}
