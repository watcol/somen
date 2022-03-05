//! Types for error handling.

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;
use core::task::Poll;
use futures_core::TryStream;

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
    T: fmt::Debug + fmt::Display,
    L: fmt::Debug,
    E: std::error::Error + 'static,
{
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parser { .. } => None,
            Self::Stream(e) => Some(e),
        }
    }
}

/// A set of expected tokens.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expects<T>(Vec<Expect<T>>);

impl<T> From<ExpectKind<T>> for Expects<T> {
    #[inline]
    fn from(inner: ExpectKind<T>) -> Self {
        Expects::from(Expect::Positive(inner))
    }
}

impl<T> From<&'static str> for Expects<T> {
    #[inline]
    fn from(msg: &'static str) -> Self {
        Expects::from(ExpectKind::Static(msg))
    }
}

impl<T> Expects<T> {
    /// Creating a new instance.
    #[inline]
    pub fn new(first: ExpectKind<T>) -> Self {
        Self::from(first)
    }

    /// Creating a new instance with [`Expect::Negative`].
    #[inline]
    pub fn new_neg(first: ExpectKind<T>) -> Self {
        Self::from(Expect::Negative(first))
    }

    /// Negate all elements.
    #[inline]
    pub fn negate(self) -> Self {
        self.map(Expect::negate)
    }

    /// Converting variant [`ExpectKind::Token`] of each elements.
    #[inline]
    pub fn map_tokens<F: FnMut(T) -> U, U>(self, mut f: F) -> Expects<U> {
        self.map(|e| e.map_token(&mut f))
    }
}

#[cfg(feature = "alloc")]
impl<T> From<Expect<T>> for Expects<T> {
    #[inline]
    fn from(inner: Expect<T>) -> Self {
        Self(alloc::vec![inner])
    }
}

#[cfg(feature = "alloc")]
impl<T> From<String> for Expects<T> {
    #[inline]
    fn from(msg: String) -> Self {
        Expects::from(ExpectKind::Owned(msg))
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
                    write!(f, " or {}", i)?;
                } else {
                    write!(f, ", {}", i)?;
                }
            }
            Ok(())
        }
    }
}

#[cfg(feature = "alloc")]
impl<T> FromIterator<Expect<T>> for Expects<T> {
    fn from_iter<I: IntoIterator<Item = Expect<T>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(feature = "alloc")]
impl<T> IntoIterator for Expects<T> {
    type Item = Expect<T>;
    type IntoIter = alloc::vec::IntoIter<Expect<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(feature = "alloc")]
impl<T> Expects<T> {
    /// Merge two sets.
    pub fn merge(mut self, mut other: Expects<T>) -> Self {
        self.0.append(&mut other.0);
        self
    }

    /// Converting each elements.
    pub fn map<F: FnMut(Expect<T>) -> Expect<U>, U>(self, mut f: F) -> Expects<U> {
        Expects(self.into_iter().map(&mut f).collect())
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

#[cfg(not(feature = "alloc"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expects<T>(Expect<T>);

#[cfg(not(feature = "alloc"))]
impl<T> From<Expect<T>> for Expects<T> {
    #[inline]
    fn from(inner: Expect<T>) -> Self {
        Self(inner)
    }
}

#[cfg(not(feature = "alloc"))]
impl<T: fmt::Display> fmt::Display for Expects<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(not(feature = "alloc"))]
impl<T> IntoIterator for Expects<T> {
    type Item = Expect<T>;
    type IntoIter = core::iter::Once<Expect<T>>;

    fn into_iter(self) -> Self::IntoIter {
        core::iter::once(self.0)
    }
}

#[cfg(not(feature = "alloc"))]
impl<T> FromIterator<Expect<T>> for Expects<T> {
    fn from_iter<I: IntoIterator<Item = Expect<T>>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        match iter.next() {
            Some(ex) if iter.next().is_none() => Self(ex),
            _ => Self(Expect::Positive(ExpectKind::Other)),
        }
    }
}

#[cfg(not(feature = "alloc"))]
impl<T> Expects<T> {
    #[inline]
    #[allow(unused_variables)]
    pub fn merge(self, other: Expects<T>) -> Self {
        Self(Expect::Positive(ExpectKind::Other))
    }

    /// Converting each elements.
    #[inline]
    pub fn map<F: FnMut(Expect<T>) -> Expect<U>, U>(self, mut f: F) -> Expects<U> {
        Expects(f(self.0))
    }

    #[inline]
    pub fn sort(&mut self)
    where
        T: Ord,
    {
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expect<T> {
    Positive(ExpectKind<T>),
    Negative(ExpectKind<T>),
}

impl<T: fmt::Display> fmt::Display for Expect<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negative(ExpectKind::Any) => write!(f, "EOF"),
            #[cfg(not(feature = "alloc"))]
            Self::Negative(ExpectKind::Other) => write!(f, "something"),
            Self::Negative(kind) => write!(f, "not {}", kind),
            Self::Positive(kind) => kind.fmt(f),
        }
    }
}

impl<T> Expect<T> {
    /// Negate the element.
    pub fn negate(self) -> Self {
        match self {
            Self::Positive(inner) => Self::Negative(inner),
            Self::Negative(inner) => Self::Positive(inner),
        }
    }

    /// Converting the inner [`ExpectKind`].
    #[inline]
    pub fn map<F, U>(self, f: F) -> Expect<U>
    where
        F: FnOnce(ExpectKind<T>) -> ExpectKind<U>,
    {
        match self {
            Self::Positive(inner) => Expect::Positive(f(inner)),
            Self::Negative(inner) => Expect::Negative(f(inner)),
        }
    }

    /// Converting the value of inner [`ExpectKind::Token`]
    #[inline]
    pub fn map_token<F, U>(self, f: F) -> Expect<U>
    where
        F: FnOnce(T) -> U,
    {
        self.map(|inner| inner.map_token(f))
    }
}

/// A value to express what tokens are expected by the parser.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExpectKind<T> {
    /// Any token.
    Any,
    /// A token.
    Token(T),
    /// A described tokens.
    Static(&'static str),
    /// A described tokens. (dynamic)
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    Owned(String),
    /// Tokens can't be expressed in `#![no_std]` environment without allocators.
    #[cfg(any(doc, not(feature = "alloc")))]
    #[cfg_attr(feature = "nightly", doc(cfg(not(feature = "alloc"))))]
    Other,
}

impl<T: fmt::Display> fmt::Display for ExpectKind<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => write!(f, "a token"),
            Self::Token(t) => t.fmt(f),
            Self::Static(s) => s.fmt(f),
            #[cfg(feature = "alloc")]
            Self::Owned(s) => s.fmt(f),
            #[cfg(not(feature = "alloc"))]
            Self::Other => write!(f, "something"),
        }
    }
}

impl<T> ExpectKind<T> {
    /// Converting the value of variant [`Token`]
    ///
    /// [`Token`]: Self::Token
    pub fn map_token<F, U>(self, f: F) -> ExpectKind<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Any => ExpectKind::Any,
            Self::Token(t) => ExpectKind::Token(f(t)),
            Self::Static(s) => ExpectKind::Static(s),
            #[cfg(feature = "alloc")]
            Self::Owned(s) => ExpectKind::Owned(s),
            #[cfg(not(feature = "alloc"))]
            Self::Other => ExpectKind::Other,
        }
    }
}
