use core::fmt;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, ParseResult};
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

#[cfg(feature = "std")]
impl std::error::Error for AnyError {}

impl<I: Positioned + ?Sized> Parser<I> for Any<I> {
    type Output = I::Ok;
    type Error = AnyError;
    type State = ();

    fn poll_parse(
        &self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        _state: &mut Self::State,
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
