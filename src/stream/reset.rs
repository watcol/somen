mod future;
use future::{MarkFuture, ResetFuture};

use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::TryStream;

/// A stream that can mark current position, and rewind its position to the mark.
pub trait Reset: TryStream {
    /// The type of markers.
    type Marker;

    /// Marking current position, and return a marker.
    fn poll_mark(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Marker, Self::Error>>;

    /// Rewinding the postion to the marker.
    fn poll_reset(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        marker: Self::Marker,
    ) -> Poll<Result<(), Self::Error>>;

    /// An asynchronous version of `poll_mark`, which returns a `Future` object.
    #[inline]
    fn mark(&mut self) -> MarkFuture<'_, Self>
    where
        Self: Unpin,
    {
        MarkFuture::new(self)
    }

    /// An asynchronous version of `poll_rewind`, which returns a `Future` object.
    #[inline]
    fn reset(&mut self, marker: Self::Marker) -> ResetFuture<'_, Self>
    where
        Self: Unpin,
        Self::Marker: Clone,
    {
        ResetFuture::new(self, marker)
    }
}
