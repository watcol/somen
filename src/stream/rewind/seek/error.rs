use core::fmt;
use futures_io::Error;

/// An error type for [`SeekRewinder`].
///
/// [`SeekRewinder`]: crate::stream::rewind::SeekRewinder
#[derive(Debug)]
pub enum SeekError<S> {
    Stream(S),
    Seek(Error),
}

impl<S: fmt::Display> fmt::Display for SeekError<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stream(e) => write!(f, "{}", e),
            Self::Seek(e) => write!(f, "{}", e),
        }
    }
}

#[cfg(feature = "std")]
impl<S: std::error::Error + 'static> std::error::Error for SeekError<S> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Stream(e) => Some(e),
            Self::Seek(e) => Some(e),
        }
    }
}
