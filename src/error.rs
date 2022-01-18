//! Types for error handling.

use core::ops::Range;
use core::{convert::Infallible, fmt};
use futures_core::stream::TryStream;

use crate::parser::Parser;
use crate::stream::position::Positioned;

/// The Result type for [`poll_parse`].
///
/// [`poll_parse`]: crate::parser::Parser::poll_parse
pub type ParseResult<P, I> = core::result::Result<
    <P as Parser<I>>::Output,
    ParseError<<P as Parser<I>>::Error, <I as TryStream>::Error>,
>;

/// The Result type for [`poll_parse_positioned`].
///
/// [`poll_parse_positioned`]: crate::parser::Parser::poll_parse_positioned
pub type PositionedResult<P, I> = core::result::Result<
    <P as Parser<I>>::Output,
    ParseError<
        PositionedError<<P as Parser<I>>::Error, <I as Positioned>::Position>,
        <I as TryStream>::Error,
    >,
>;

/// The error type for this crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError<P, S = Infallible> {
    /// A parsing error. (with position)
    Parser(P),
    /// An error while reading streams.
    Stream(S),
}

impl<P: fmt::Display, S: fmt::Display> fmt::Display for ParseError<P, S> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parser(e) => write!(f, "{}", e),
            Self::Stream(e) => write!(f, "{}", e),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
impl<P, S> std::error::Error for ParseError<P, S>
where
    P: std::error::Error + 'static,
    S: std::error::Error + 'static,
{
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parser(e) => Some(e),
            Self::Stream(e) => Some(e),
        }
    }
}

/// A parsing error with information about where the error occured.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionedError<E, P> {
    /// The position where the error occured.
    pub range: Range<P>,
    /// The original error.
    pub error: E,
}

impl<E: fmt::Display, P> fmt::Display for PositionedError<E, P> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}

#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
impl<E: std::error::Error + 'static, P: fmt::Debug> std::error::Error for PositionedError<E, P> {
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
