//! Types for error handling.

use core::ops::Range;
use core::{convert::Infallible, fmt};

/// The error type for this crate.
#[derive(Debug)]
pub enum Error<P, S = Infallible> {
    /// A parsing error. (with position)
    Parse(P),
    /// An error while reading streams.
    Stream(S),
}

impl<P: fmt::Display, S: fmt::Display> fmt::Display for Error<P, S> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{}", e),
            Self::Stream(e) => write!(f, "{}", e),
        }
    }
}

/// A trait for parsing errors.
pub trait ParseError: fmt::Display {
    /// The returning position type.
    type Position;

    /// The range where the error was occured.
    fn range(&self) -> Range<Self::Position>;
}

#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
impl<P, S> std::error::Error for Error<P, S>
where
    P: std::error::Error + 'static,
    S: std::error::Error + 'static,
{
    #[inline]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(e) => Some(e),
            Self::Stream(e) => Some(e),
        }
    }
}
