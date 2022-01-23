use core::fmt;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`any`].
///
/// [`any`]: crate::parser::any
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Any;

impl Any {
    /// Creating a new instance.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

/// An error type for parser [`any`].
///
/// [`any`]: crate::parser::any
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnyError;

impl fmt::Display for AnyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected EOF")
    }
}

impl<I: Positioned + ?Sized> Parser<I> for Any {
    type Output = I::Ok;
    type Error = AnyError;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
    ) -> Poll<ParseResult<Self, I>> {
        let start = input.position();
        let parsed = ready!(input
            .as_mut()
            .try_poll_next(cx)
            .map_err(ParseError::Stream)?);
        Poll::Ready(match parsed {
            Some(i) => Ok(i),
            None => Err(ParseError::Parser(AnyError, start..input.position())),
        })
    }
}
