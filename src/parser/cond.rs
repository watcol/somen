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
pub struct Cond<I: ?Sized, F> {
    cond: F,
    _phantom: PhantomData<I>,
}

impl<I: ?Sized, F> Cond<I, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(cond: F) -> Self {
        Self {
            cond,
            _phantom: PhantomData,
        }
    }
}

impl<I, F> Parser<I> for Cond<I, F>
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
