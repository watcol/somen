use core::fmt;

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
    Owned(alloc::string::String),
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
