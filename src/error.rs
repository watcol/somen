use crate::position::{NopPosition, Step};
use core::{convert::Infallible, fmt, ops::Range};

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, string::String};

#[cfg(feature = "std")]
#[derive(Debug)]
pub struct Error<P: Step = NopPosition, S: std::error::Error = Infallible> {
    pub range: Range<P>,
    pub kind: ErrorKind<S>,
}

#[cfg(not(feature = "std"))]
pub struct Error<P: Step = NopPosition, S: fmt::Display = Infallible> {
    pub range: Range<P>,
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
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}

#[cfg(feature = "std")]
#[derive(Debug)]
pub enum ErrorKind<S: std::error::Error = Infallible> {
    Expected { expected: String },
    Conversion { inner: Box<dyn std::error::Error> },
    Streaming { inner: S },
}

#[cfg(not(feature = "std"))]
#[derive(Debug)]
pub enum ErrorKind<S: fmt::Display = Infallible> {
    Expected {
        #[cfg(feature = "alloc")]
        expected: String,
    },
    Conversion {
        #[cfg(feature = "alloc")]
        inner: Box<dyn fmt::Display>,
    },
    Streaming {
        inner: S,
    },
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "alloc")]
            Self::Expected { expected } => write!(f, "expected {}", expected),
            #[cfg(not(feature = "alloc"))]
            Self::Expected {} => write!(f, "expected something"),
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
