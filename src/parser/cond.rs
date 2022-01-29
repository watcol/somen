use core::fmt;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`is`].
///
/// [`is`]: crate::parser::is
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// An error type for parser [`is`].
///
/// [`is`]: crate::parser::is
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CondError;

impl fmt::Display for CondError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected a value matches the condition")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CondError {}

impl<I, F> Parser<I> for Cond<I, F>
where
    I: Positioned + ?Sized,
    F: Fn(&I::Ok) -> bool,
{
    type Output = I::Ok;
    type Error = CondError;
    type State = ();

    fn poll_parse(
        &self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        _state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        let start = input.position();
        let parsed = ready!(input.as_mut().try_poll_next(cx)?);
        Poll::Ready(match parsed {
            Some(i) if (self.cond)(&i) => Ok(i),
            _ => Err(ParseError::Parser(CondError, start..input.position())),
        })
    }
}
