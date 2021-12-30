use core::fmt;

/// An error type by [`BufferedStream`].
///
/// [`BufferedStream`]: crate::stream::BufferedStream
#[derive(Debug)]
pub enum BufferedError<S> {
    Stream(S),
    Buffer,
}

impl<S: fmt::Display> fmt::Display for BufferedError<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stream(e) => write!(f, "{}", e),
            Self::Buffer => write!(f, "a marker used by illegal order"),
        }
    }
}

#[cfg(feature = "std")]
impl<S: std::error::Error + 'static> std::error::Error for BufferedError<S> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Stream(e) => Some(e),
            Self::Buffer => None,
        }
    }
}
