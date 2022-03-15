//! Types for error handling.

mod expects;

use core::fmt;
use core::ops::Range;
use core::task::Poll;
use futures_core::TryStream;

pub use expects::*;

use crate::stream::Positioned;

/// The Result type for [`poll_parse`].
///
/// [`poll_parse`]: crate::parser::Parser::poll_parse
pub type PolledResult<O, I> = Poll<
    Result<
        (
            Status<O, <I as TryStream>::Ok, <I as Positioned>::Locator>,
            Range<<I as Positioned>::Locator>,
        ),
        <I as TryStream>::Error,
    >,
>;

/// The Result type for [`parse`].
///
/// [`parse`]: crate::parser::ParserExt::parse
pub type ParseResult<O, I> = Result<
    O,
    ParseError<<I as TryStream>::Ok, <I as Positioned>::Locator, <I as TryStream>::Error>,
>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status<O, T, L> {
    Success(O, Option<Error<T, L>>),
    Failure(Error<T, L>, bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error<T, L> {
    pub expects: Expects<T>,
    pub position: Range<L>,
}

impl<T, L> Error<T, L> {
    /// Check if the parser can rewind input to `pos` discarding this error, or not.
    #[inline]
    pub fn rewindable(&self, pos: &L) -> bool
    where
        L: PartialEq,
    {
        self.position.start == *pos
    }

    /// Sort and remove duplicates in the expected tokens.
    #[inline]
    pub fn sort_expects(&mut self)
    where
        T: Ord,
    {
        self.expects.sort()
    }

    /// Converting [`ExpectKind::Token`] of each expects.
    #[inline]
    pub fn map_tokens<F: FnMut(T) -> U, U>(self, f: F) -> Error<U, L> {
        Error {
            expects: self.expects.map_tokens(f),
            position: self.position,
        }
    }
}

impl<T: fmt::Display, L> fmt::Display for Error<T, L> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected {}.", self.expects)
    }
}

#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
impl<T: fmt::Debug + fmt::Display, L: fmt::Debug> std::error::Error for Error<T, L> {}

/// The error type for this crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError<T, L, E> {
    /// A parsing error.
    Parser(Error<T, L>),
    /// An error while reading streams.
    Stream(E),
}

impl<T, L, E> From<E> for ParseError<T, L, E> {
    #[inline]
    fn from(error: E) -> Self {
        Self::Stream(error)
    }
}

impl<T, U, L, E> ParseError<T, L, ParseError<U, L, E>> {
    pub fn flatten(self) -> ParseError<Result<T, U>, L, E> {
        match self {
            Self::Parser(err) => ParseError::Parser(err.map_tokens(Ok)),
            Self::Stream(ParseError::Parser(err)) => ParseError::Parser(err.map_tokens(Err)),
            Self::Stream(ParseError::Stream(e)) => ParseError::Stream(e),
        }
    }
}

impl<T: fmt::Display, L, E: fmt::Display> fmt::Display for ParseError<T, L, E> {
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
impl<T, L, E> std::error::Error for ParseError<T, L, E>
where
    T: fmt::Debug + fmt::Display + 'static,
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
