//! Rewinding streams.

#[cfg(feature = "alloc")]
mod buffered;

use core::pin::Pin;

#[cfg(feature = "alloc")]
pub use buffered::{BufferedError, BufferedRewinder};

use futures_core::TryStream;

/// A stream that can mark current position, and rewind its position to the mark.
pub trait Rewind: TryStream {
    /// The type of markers.
    type Marker;

    /// Marking current position, and return a marker.
    fn mark(self: Pin<&mut Self>) -> Result<Self::Marker, Self::Error>;

    /// Rewinding the postion to the marker.
    ///
    /// Note that some types implement this require using from most recent generated marker.
    fn rewind(self: Pin<&mut Self>, marker: Self::Marker) -> Result<(), Self::Error>;

    /// Dropping unused markers.
    ///
    /// Users can use it for explicitly declare as the marker will no longer be used.
    #[allow(unused_variables)]
    #[inline]
    fn drop_marker(self: Pin<&mut Self>, marker: Self::Marker) -> Result<(), Self::Error> {
        Ok(())
    }
}
