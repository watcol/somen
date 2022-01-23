//! Rewinding streams.

#[cfg(feature = "alloc")]
mod buffered;
mod future;
#[cfg(feature = "std")]
mod seek;

#[cfg(feature = "alloc")]
pub use buffered::{BufferedError, BufferedRewinder};
#[cfg(feature = "std")]
pub use seek::{SeekError, SeekRewinder};

use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::TryStream;

use future::{MarkFuture, RewindFuture};

/// A stream that can mark current position, and rewind its position to the mark.
pub trait Rewind: TryStream {
    /// The type of markers.
    type Marker;

    /// Marking current position, and return a marker.
    fn poll_mark(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Marker, Self::Error>>;

    /// Rewinding the postion to the marker.
    ///
    /// Note that some types implement this require using from most recent generated marker.
    fn poll_rewind(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        marker: Self::Marker,
    ) -> Poll<Result<(), Self::Error>>;

    /// Dropping unused markers.
    ///
    /// Users can use it for explicitly declare as the marker will no longer be used.
    #[allow(unused_variables)]
    #[inline]
    fn drop_marker(&mut self, marker: Self::Marker) -> Result<(), Self::Error> {
        Ok(())
    }

    /// An asynchronous version of [`poll_mark`], which returns a [`Future`] object.
    ///
    /// [`poll_mark`]: Self::poll_mark
    /// [`Future`]: core::future::Future
    #[inline]
    fn mark(&mut self) -> MarkFuture<'_, Self>
    where
        Self: Unpin,
    {
        MarkFuture::new(self)
    }

    /// An asynchronous version of [`poll_rewind`], which returns a [`Future`] object.
    ///
    /// [`poll_rewind`]: Self::poll_rewind
    /// [`Future`]: core::future::Future
    #[inline]
    fn rewind(&mut self, marker: Self::Marker) -> RewindFuture<'_, Self>
    where
        Self: Unpin,
        Self::Marker: Clone,
    {
        RewindFuture::new(self, marker)
    }
}
