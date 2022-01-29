use core::fmt;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`eof`].
///
/// [`eof`]: crate::parser::eof
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Eof<I: ?Sized>(PhantomData<I>);

impl<I: ?Sized> Default for Eof<I> {
    fn default() -> Self {
        Eof(PhantomData)
    }
}

impl<I: ?Sized> Eof<I> {
    /// Creating a new instance.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

/// An error type for parser [`eof`].
///
/// [`eof`]: crate::parser::eof
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EofError;

impl fmt::Display for EofError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected EOF")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EofError {}

impl<I: Positioned + ?Sized> Parser<I> for Eof<I> {
    type Output = ();
    type Error = EofError;
    type State = ();

    fn poll_parse(
        &self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        _state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        let start = input.position();
        Poll::Ready(match ready!(input.as_mut().try_poll_next(cx)?) {
            Some(_) => Err(ParseError::Parser(EofError, start..input.position())),
            None => Ok(()),
        })
    }
}
