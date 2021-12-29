//! Types for error handling.

use core::{convert::Infallible, fmt, ops::Range};

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, string::String};

/// The position where an error has ocuured and the description.
#[derive(Debug)]
pub struct Error<
    P: PartialOrd + Clone = (),
    #[cfg(feature = "std")] S: std::error::Error = Infallible,
    #[cfg(not(feature = "std"))] S: fmt::Display = Infallible,
> {
    /// The range where this error has occured.
    pub range: Range<P>,
    /// The kind of this error.
    pub kind: ErrorKind<S>,
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}

/// The kinds of errors.
#[derive(Debug)]
pub enum ErrorKind<
    #[cfg(feature = "std")] S: std::error::Error = Infallible,
    #[cfg(not(feature = "std"))] S: fmt::Display = Infallible,
> {
    /// Expected something, but unmatched.
    Expected {
        #[cfg(feature = "alloc")]
        expected: String,
    },
    /// Errors while conversion by `try_map`.
    Conversion {
        #[cfg(feature = "std")]
        inner: Box<dyn std::error::Error>,
        #[cfg(all(not(feature = "std"), feature = "alloc"))]
        inner: Box<dyn fmt::Display>,
    },
    /// Errors while consumption by `TryStream`.
    Streaming { inner: S },
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "alloc")]
            Self::Expected { expected } => write!(f, "expected {}", expected),
            #[cfg(not(feature = "alloc"))]
            Self::Expected {} => write!(f, "parsing failed"),
            #[cfg(feature = "alloc")]
            Self::Conversion { inner } => write!(f, "{}", inner),
            #[cfg(not(feature = "alloc"))]
            Self::Conversion {} => write!(f, "conversion failed"),
            Self::Streaming { inner } => write!(f, "{}", inner),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Expected { .. } => None,
            Self::Conversion { inner } => Some(inner.as_ref()),
            Self::Streaming { inner } => Some(inner),
        }
    }
}
