use core::fmt;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, ParseResult};
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

/// An error type for parser [`token`].
///
/// [`token`]: crate::parser::token
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TokenError<T>(T);

impl<T: fmt::Display> fmt::Display for TokenError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected {}", self.0)
    }
}

#[cfg(feature = "std")]
impl<T: fmt::Debug + fmt::Display> std::error::Error for TokenError<T> {}

impl<I> Parser<I> for Token<I, I::Ok>
where
    I: Positioned + ?Sized,
    I::Ok: Clone + Eq,
{
    type Output = I::Ok;
    type Error = TokenError<I::Ok>;
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
            Some(i) if i == self.token => Ok(i),
            _ => Err(ParseError::Parser(
                TokenError(self.token.clone()),
                start..input.position(),
            )),
        })
    }
}
