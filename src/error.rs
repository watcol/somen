//! Types for error handling.

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;
use futures_core::TryStream;

use crate::stream::Positioned;

/// The Result type for [`parse`].
///
/// [`parse`]: crate::parser::ParserExt::parse
pub type ParseResult<O, I> = core::result::Result<
    O,
    ParseError<<I as TryStream>::Ok, <I as Positioned>::Locator, <I as TryStream>::Error>,
>;

/// The error type for this crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError<T, L, E> {
    /// A parsing error with expected tokens and the position.
    Parser(Expects<T>, Range<L>),
    /// An error while reading streams.
    Stream(E),
}

impl<T, L, E> From<E> for ParseError<T, L, E> {
    #[inline]
    fn from(error: E) -> Self {
        Self::Stream(error)
    }
}

impl<T, L, E> ParseError<T, L, E> {
    /// Sort and remove duplicates in the expected tokens.
    pub fn sort_expects(&mut self)
    where
        T: Ord,
    {
        if let ParseError::Parser(ref mut ex, _) = *self {
            ex.sort();
        }
    }
}

impl<T, U, L, E> ParseError<T, L, ParseError<U, L, E>> {
    pub fn flatten(self) -> ParseError<Result<T, U>, L, E> {
        match self {
            #[cfg(feature = "alloc")]
            Self::Parser(Expects(ex), p) => ParseError::Parser(
                Expects(ex.into_iter().map(|e| e.map_token(Ok)).collect()),
                p,
            ),
            #[cfg(not(feature = "alloc"))]
            Self::Parser(Expects(ex), p) => ParseError::Parser(Expects(ex.map_token(Ok)), p),
            #[cfg(feature = "alloc")]
            Self::Stream(ParseError::Parser(Expects(ex), p)) => ParseError::Parser(
                Expects(ex.into_iter().map(|e| e.map_token(Err)).collect()),
                p,
            ),
            #[cfg(not(feature = "alloc"))]
            Self::Stream(ParseError::Parser(Expects(ex), p)) => {
                ParseError::Parser(Expects(ex.map_token(Err)), p)
            }
            Self::Stream(ParseError::Stream(e)) => ParseError::Stream(e),
        }
    }
}

impl<T: fmt::Display, L, E: fmt::Display> fmt::Display for ParseError<T, L, E> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parser(e, _) => write!(f, "expected {}.", e),
            Self::Stream(e) => e.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
impl<T, L, E> std::error::Error for ParseError<T, L, E>
where
    T: fmt::Debug + fmt::Display,
    L: fmt::Debug,
    E: std::error::Error + 'static,
{
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parser(_, _) => None,
            Self::Stream(e) => Some(e),
        }
    }
}

/// A set of expected tokens.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expects<T>(Vec<Expect<T>>);

#[cfg(feature = "alloc")]
impl<T> FromIterator<Expect<T>> for Expects<T> {
    fn from_iter<I: IntoIterator<Item = Expect<T>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(feature = "alloc")]
impl<T> Expects<T> {
    /// Creating a new instance.
    pub fn new(first: Expect<T>) -> Self {
        Self(alloc::vec![first])
    }

    /// Merge two sets.
    pub fn merge(mut self, mut other: Expects<T>) -> Self {
        self.0.append(&mut other.0);
        self
    }

    /// Sort and remove duplicates.
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.0.sort_unstable();
        self.0.dedup();
    }
}

#[cfg(feature = "alloc")]
impl<T: fmt::Display> fmt::Display for Expects<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.0.len();
        if len == 0 {
            Ok(())
        } else {
            for (c, i) in self.0.iter().enumerate() {
                if c == 0 {
                    write!(f, "{}", i)?;
                } else if c == len - 1 {
                    write!(f, "or {}", i)?;
                } else {
                    write!(f, ", {}", i)?;
                }
            }
            Ok(())
        }
    }
}

#[cfg(not(feature = "alloc"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expects<T>(Expect<T>);

#[cfg(not(feature = "alloc"))]
impl<T> FromIterator<Expect<T>> for Expects<T> {
    fn from_iter<I: IntoIterator<Item = Expect<T>>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        match iter.next() {
            Some(ex) if iter.next().is_none() => Self(ex),
            _ => Self(Expect::Other),
        }
    }
}

#[cfg(not(feature = "alloc"))]
impl<T> Expects<T> {
    pub fn new(first: Expect<T>) -> Self {
        Self(first)
    }

    #[allow(unused_variables)]
    pub fn merge(self, other: Expects<T>) -> Self {
        Self(Expect::Other)
    }

    pub fn sort(&mut self)
    where
        T: Ord,
    {
    }
}

#[cfg(not(feature = "alloc"))]
impl<T: fmt::Display> fmt::Display for Expects<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<Expect<T>> for Expects<T> {
    fn from(inner: Expect<T>) -> Self {
        Expects::new(inner)
    }
}

impl<T> From<&'static str> for Expects<T> {
    fn from(msg: &'static str) -> Self {
        Expects::new(Expect::Static(msg))
    }
}

#[cfg(feature = "alloc")]
impl<T> From<String> for Expects<T> {
    fn from(msg: String) -> Self {
        Expects::new(Expect::Owned(msg))
    }
}

/// A value to express what tokens are expected by the parser.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expect<T> {
    /// A token.
    Token(T),
    /// A described tokens.
    Static(&'static str),
    /// A described tokens. (dynamic)
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    Owned(String),
    /// The end of input.
    Eof,
    /// Tokens can't be expressed in `#![no_std]` environment without allocators.
    #[cfg(any(doc, not(feature = "alloc")))]
    #[cfg_attr(feature = "nightly", doc(cfg(not(feature = "alloc"))))]
    Other,
}

impl<T: fmt::Display> fmt::Display for Expect<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Token(t) => t.fmt(f),
            Self::Static(s) => s.fmt(f),
            #[cfg(feature = "alloc")]
            Self::Owned(s) => s.fmt(f),
            Self::Eof => write!(f, "EOF"),
            #[cfg(not(feature = "alloc"))]
            Self::Other => write!(f, "something"),
        }
    }
}

impl<T> Expect<T> {
    /// Converting the value of variant [`Token`]
    ///
    /// [`Token`]: Self::Token
    pub fn map_token<F, U>(self, f: F) -> Expect<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Token(t) => Expect::Token(f(t)),
            Self::Static(s) => Expect::Static(s),
            #[cfg(feature = "alloc")]
            Self::Owned(s) => Expect::Owned(s),
            Self::Eof => Expect::Eof,
            #[cfg(not(feature = "alloc"))]
            Self::Other => Expect::Other,
        }
    }
}
