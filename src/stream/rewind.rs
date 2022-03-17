//! Rewinds streams.

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

    /// Marks current position, and return a marker.
    fn mark(self: Pin<&mut Self>) -> Result<Self::Marker, Self::Error>;

    /// Rewinds the postion to the marker.
    ///
    /// Note that some types implement this require using from most recent generated marker.
    fn rewind(self: Pin<&mut Self>, marker: Self::Marker) -> Result<(), Self::Error>;

    /// Drops unused markers.
    ///
    /// Users can use it for explicitly declare as the marker will no longer be used.
    #[allow(unused_variables)]
    #[inline]
    fn drop_marker(self: Pin<&mut Self>, marker: Self::Marker) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Performs [`mark`] for unpinned stream.
    ///
    /// [`mark`]: Self::mark
    #[inline]
    fn mark_unpin(&mut self) -> Result<Self::Marker, Self::Error>
    where
        Self: Unpin,
    {
        Pin::new(&mut *self).mark()
    }

    /// Performs [`rewind`] for unpinned stream.
    ///
    /// [`rewind`]: Self::rewind
    #[inline]
    fn rewind_unpin(&mut self, marker: Self::Marker) -> Result<(), Self::Error>
    where
        Self: Unpin,
    {
        Pin::new(&mut *self).rewind(marker)
    }

    /// Performs [`drop_marker`] for unpinned stream.
    ///
    /// [`drop_marker`]: Self::drop_marker
    #[inline]
    fn drop_marker_unpin(&mut self, marker: Self::Marker) -> Result<(), Self::Error>
    where
        Self: Unpin,
    {
        Pin::new(&mut *self).drop_marker(marker)
    }
}
