use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, Expect, Expects, PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`is`].
///
/// [`is`]: crate::parser::is
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Is<I: ?Sized, F> {
    cond: F,
    _phantom: PhantomData<I>,
}

impl<I: ?Sized, F> Is<I, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(cond: F) -> Self {
        Self {
            cond,
            _phantom: PhantomData,
        }
    }
}

impl<I, F> Parser<I> for Is<I, F>
where
    I: Positioned + ?Sized,
    F: FnMut(&I::Ok) -> bool,
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
        let res = ready!(input.as_mut().try_poll_next(cx)?);
        let end = input.position();
        Poll::Ready(Ok((
            match res {
                Some(val) if (self.cond)(&val) => Status::Success(val, None),
                _ => Status::Failure(
                    Error {
                        expects: Expects::new(Expect::Static("<cond>")),
                        position: start.clone()..end.clone(),
                    },
                    false,
                ),
            },
            start..end,
        )))
    }
}

/// A parser for function [`is_not`].
///
/// [`is_not`]: crate::parser::is_not
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsNot<I: ?Sized, F> {
    cond: F,
    _phantom: PhantomData<I>,
}

impl<I: ?Sized, F> IsNot<I, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(cond: F) -> Self {
        Self {
            cond,
            _phantom: PhantomData,
        }
    }
}

impl<I, F> Parser<I> for IsNot<I, F>
where
    I: Positioned + ?Sized,
    F: FnMut(&I::Ok) -> bool,
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
        let res = ready!(input.as_mut().try_poll_next(cx)?);
        let end = input.position();
        Poll::Ready(Ok((
            match res {
                Some(val) if !(self.cond)(&val) => Status::Success(val, None),
                _ => Status::Failure(
                    Error {
                        expects: Expects::new(Expect::Static("<cond>")),
                        position: start.clone()..end.clone(),
                    },
                    false,
                ),
            },
            start..end,
        )))
    }
}

/// A parser for function [`is_some`].
///
/// [`is_some`]: crate::parser::is_some
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsSome<I: ?Sized, F> {
    cond: F,
    _phantom: PhantomData<I>,
}

impl<I: ?Sized, F> IsSome<I, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(cond: F) -> Self {
        Self {
            cond,
            _phantom: PhantomData,
        }
    }
}

impl<I, F, O> Parser<I> for IsSome<I, F>
where
    I: Positioned + ?Sized,
    F: FnMut(&I::Ok) -> Option<O>,
{
    type Output = O;
    type State = ();

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        _state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        let start = input.position();
        let res = ready!(input.as_mut().try_poll_next(cx)?);
        let end = input.position();
        Poll::Ready(Ok((
            match res {
                // TODO: fix it on "if_let_guard" are stabilized.
                Some(i) => match (self.cond)(&i) {
                    Some(val) => Status::Success(val, None),
                    None => Status::Failure(
                        Error {
                            expects: Expects::new(Expect::Static("<some>")),
                            position: start.clone()..end.clone(),
                        },
                        false,
                    ),
                },
                _ => Status::Failure(
                    Error {
                        expects: Expects::new(Expect::Static("<some>")),
                        position: start.clone()..end.clone(),
                    },
                    false,
                ),
            },
            start..end,
        )))
    }
}
