use core::fmt;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`map`].
///
/// [`map`]: super::Parser::map
#[derive(Debug)]
pub struct Map<P, F> {
    inner: P,
    f: F,
}

impl<P, F> Map<P, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, F, I, O> Parser<I> for Map<P, F>
where
    P: Parser<I>,
    F: Fn(P::Output) -> O,
    I: Positioned + ?Sized,
{
    type Output = O;
    type Error = P::Error;
    type State = P::State;

    fn poll_parse(
        &self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        self.inner.poll_parse(input, cx, state).map_ok(&self.f)
    }
}

/// A parser for method [`try_map`].
///
/// [`try_map`]: super::Parser::try_map
#[derive(Debug)]
pub struct TryMap<P, F> {
    inner: P,
    f: F,
}

impl<P, F> TryMap<P, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

/// An error type for method [`try_map`].
///
/// [`try_map`]: super::Parser::try_map
#[derive(Debug)]
pub enum TryMapError<E, F> {
    Parser(E),
    Conversion(F),
}

impl<E: fmt::Display, F: fmt::Display> fmt::Display for TryMapError<E, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parser(e) => e.fmt(f),
            Self::Conversion(e) => e.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
impl<E, F> std::error::Error for TryMapError<E, F>
where
    E: std::error::Error + 'static,
    F: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parser(e) => Some(e),
            Self::Conversion(e) => Some(e),
        }
    }
}

impl<P, F, I, O, E> Parser<I> for TryMap<P, F>
where
    P: Parser<I>,
    F: Fn(P::Output) -> Result<O, E>,
    I: Positioned + ?Sized,
{
    type Output = O;
    type Error = TryMapError<P::Error, E>;
    type State = P::State;

    fn poll_parse(
        &self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        let start = input.position();
        Poll::Ready(
            match ready!(self.inner.poll_parse(input.as_mut(), cx, state)) {
                Ok(x) => match (self.f)(x) {
                    Ok(x) => Ok(x),
                    Err(e) => Err(ParseError::Fatal(
                        TryMapError::Conversion(e),
                        start..input.position(),
                    )),
                },
                Err(err) => Err(err.map_parse(TryMapError::Parser)),
            },
        )
    }
}
