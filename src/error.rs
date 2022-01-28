//! Types for error handling.

use core::fmt;
use core::ops::Range;
use futures_core::TryStream;

use crate::parser::streamed::StreamedParser;
use crate::parser::Parser;
use crate::stream::Positioned;

/// The Result type for [`parse`].
///
/// [`parse`]: crate::parser::Parser::parse
pub type ParseResult<P, I> = core::result::Result<
    <P as Parser<I>>::Output,
    ParseError<<P as Parser<I>>::Error, <I as TryStream>::Error, <I as Positioned>::Locator>,
>;

/// The Result type for [`parse_streamed`].
///
/// [`parse_streamed`]: crate::parser::streamed::StreamedParser::parse_streamed
pub type StreamedResult<P, I> = core::result::Result<
    Option<<P as StreamedParser<I>>::Item>,
    ParseError<
        <P as StreamedParser<I>>::Error,
        <I as TryStream>::Error,
        <I as Positioned>::Locator,
    >,
>;

/// The error type for this crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError<E, F, L> {
    /// A parsing error. (with position)
    Parser(E, Range<L>),
    /// An error while reading streams.
    Stream(F),
}

impl<E, F, L> From<F> for ParseError<E, F, L> {
    #[inline]
    fn from(error: F) -> Self {
        Self::Stream(error)
    }
}

impl<E, F, L> ParseError<E, F, L> {
    /// Converting the inner error in the variant [`Parser`] into another type.
    ///
    /// [`Parser`]: Self::Parser
    #[inline]
    pub fn map_parse<U, Fun>(self, f: Fun) -> ParseError<U, F, L>
    where
        Fun: FnOnce(E) -> U,
    {
        match self {
            Self::Parser(e, p) => ParseError::Parser(f(e), p),
            Self::Stream(e) => ParseError::Stream(e),
        }
    }
}

impl<E: fmt::Display, F: fmt::Display, L> fmt::Display for ParseError<E, F, L> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parser(e, _) => e.fmt(f),
            Self::Stream(e) => e.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
impl<E, F, P> std::error::Error for ParseError<E, F, P>
where
    E: std::error::Error + 'static,
    F: std::error::Error + 'static,
    P: fmt::Debug,
{
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parser(e, _) => Some(e),
            Self::Stream(e) => Some(e),
        }
    }
}
