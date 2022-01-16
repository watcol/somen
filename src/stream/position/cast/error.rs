use core::fmt;

/// An error type for [`CastPositioner`].
///
/// [`CastPositioner`]: crate::stream::position::CastPositioner
#[derive(Debug)]
pub enum CastError<S, T> {
    Stream(S),
    Convert(T),
}

impl<S: fmt::Display, T: fmt::Display> fmt::Display for CastError<S, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stream(e) => write!(f, "{}", e),
            Self::Convert(e) => write!(f, "{}", e),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
impl<S, T> std::error::Error for CastError<S, T>
where
    S: std::error::Error + 'static,
    T: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Stream(e) => Some(e),
            Self::Convert(e) => Some(e),
        }
    }
}
