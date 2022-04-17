//! Types for error handling.

mod expects;

use core::fmt;
use core::ops::Range;
use core::task::Poll;
use futures_core::TryStream;

pub use expects::*;

use crate::stream::Positioned;

/// The result type for [`poll_parse`].
///
/// [`poll_parse`]: crate::parser::Parser::poll_parse
pub type PolledResult<O, I> =
    Poll<Result<Status<O, <I as Positioned>::Locator>, <I as TryStream>::Error>>;

/// The result type for [`parse`].
///
/// [`parse`]: crate::parser::ParserExt::parse
pub type ParseResult<O, I> =
    Result<O, ParseError<<I as Positioned>::Locator, <I as TryStream>::Error>>;

/// The parsed status for method [`poll_parse`].
///
/// [`poll_parse`]: crate::parser::Parser::poll_parse
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status<O, L> {
    /// Succeeded parsing.
    ///
    /// If the second elements is [`Some`], it represents that an error has occured, but the parser
    /// discarded the error by rewinding the input stream.
    Success(O, Option<Error<L>>),

    /// Failed parsing.
    ///
    /// If the second elements is `true`, it represents that this error is exclusive and merging
    /// other errors are disallowed.
    Failure(Error<L>, bool),
}

/// Errors while parsing streams.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error<L> {
    /// Expected tokens.
    pub expects: Expects,

    /// The position where the error has occured.
    pub position: Range<L>,
}

impl<L> Error<L> {
    /// Checks if the parser can rewind input to `pos` discarding this error, or not.
    #[inline]
    pub fn rewindable(&self, pos: &L) -> bool
    where
        L: PartialEq,
    {
        self.position.start == *pos
    }
}

impl<L> fmt::Display for Error<L> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected {}.", self.expects)
    }
}

#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
impl<L: fmt::Debug> std::error::Error for Error<L> {}

/// The error type for this crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError<L, E> {
    /// A parsing error.
    Parser(Error<L>),

    /// An error while reading streams.
    Stream(E),
}

impl<L, E> From<E> for ParseError<L, E> {
    #[inline]
    fn from(error: E) -> Self {
        Self::Stream(error)
    }
}

impl<L, E> ParseError<L, ParseError<L, E>> {
    pub fn flatten(self) -> ParseError<L, E> {
        match self {
            Self::Parser(err) | Self::Stream(ParseError::Parser(err)) => ParseError::Parser(err),
            Self::Stream(ParseError::Stream(e)) => ParseError::Stream(e),
        }
    }
}

impl<L, E: fmt::Display> fmt::Display for ParseError<L, E> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parser(err) => err.fmt(f),
            Self::Stream(e) => e.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
impl<L, E> std::error::Error for ParseError<L, E>
where
    L: fmt::Debug + 'static,
    E: std::error::Error + 'static,
{
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parser(e) => Some(e),
            Self::Stream(e) => Some(e),
        }
    }
}
