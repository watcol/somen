use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Expect, Expects, ParseError, ParseResult, Tracker};
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
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        let start = input.position();
        let parsed = ready!(input.as_mut().try_poll_next(cx)?);
        Poll::Ready(match parsed {
            Some(i) if (self.cond)(&i) => {
                tracker.clear();
                Ok(i)
            }
            _ => Err(ParseError::Parser {
                expects: Expects::new(Expect::Static("<condition>")),
                position: start..input.position(),
                fatal: false,
            }),
        })
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
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        let start = input.position();
        let parsed = ready!(input.as_mut().try_poll_next(cx)?);
        Poll::Ready(match parsed {
            Some(i) if !(self.cond)(&i) => {
                tracker.clear();
                Ok(i)
            }
            _ => Err(ParseError::Parser {
                expects: Expects::new(Expect::Static("<condition>")),
                position: start..input.position(),
                fatal: false,
            }),
        })
    }
}

/// A parser for function [`is_some`].
///
/// [`is_some`]: crate::parser::is_some
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CondMap<I: ?Sized, F> {
    cond: F,
    _phantom: PhantomData<I>,
}

impl<I: ?Sized, F> CondMap<I, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(cond: F) -> Self {
        Self {
            cond,
            _phantom: PhantomData,
        }
    }
}

impl<I, F, O> Parser<I> for CondMap<I, F>
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
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        let start = input.position();
        let parsed = ready!(input.as_mut().try_poll_next(cx)?);
        Poll::Ready(match parsed {
            // TODO: fix it on "if_let_guard" are stabilized.
            Some(i) => match (self.cond)(&i) {
                Some(res) => {
                    tracker.clear();
                    Ok(res)
                }
                None => Err(ParseError::Parser {
                    expects: Expects::new(Expect::Static("<some>")),
                    position: start..input.position(),
                    fatal: false,
                }),
            },
            _ => Err(ParseError::Parser {
                expects: Expects::new(Expect::Static("<some>")),
                position: start..input.position(),
                fatal: false,
            }),
        })
    }
}
