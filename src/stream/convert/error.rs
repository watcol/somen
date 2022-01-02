use core::fmt;

/// An error type for [`ConvertedStream`] and [`ConvertedRecorder`].
///
/// [`ConvertedStream`]: crate::stream::convert::ConvertedStream
/// [`ConvertedRecorder`]: crate::stream::convert::ConvertedRecorder
#[derive(Debug)]
pub enum ConvertError<S, C> {
    Stream(S),
    Conversion(C),
}

impl<S: fmt::Display, C: fmt::Display> fmt::Display for ConvertError<S, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConvertError::Stream(e) => write!(f, "{}", e),
            ConvertError::Conversion(e) => write!(f, "{}", e),
        }
    }
}

#[cfg(feature = "std")]
impl<S, C> std::error::Error for ConvertError<S, C>
where
    S: std::error::Error + 'static,
    C: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConvertError::Stream(e) => Some(e),
            ConvertError::Conversion(e) => Some(e),
        }
    }
}
