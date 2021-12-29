mod future;
use future::{MarkFuture, RewindFuture};

use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::TryStream;

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
    fn poll_rewind(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        marker: Self::Marker,
    ) -> Poll<Result<(), Self::Error>>;

    /// An asynchronous version of [`poll_mark`](#tymethod.poll_mark), which returns a
    /// [`Future`](https://doc.rust-lang.org/stable/std/future/trait.Future.html) object.
    #[inline]
    fn mark(&mut self) -> MarkFuture<'_, Self>
    where
        Self: Unpin,
    {
        MarkFuture::new(self)
    }

    /// An asynchronous version of [`poll_rewind`](#tymethod.poll_rewind), which returns a
    /// [`Future`](https://doc.rust-lang.org/stable/std/future/trait.Future.html) object.
    #[inline]
    fn rewind(&mut self, marker: Self::Marker) -> RewindFuture<'_, Self>
    where
        Self: Unpin,
        Self::Marker: Clone,
    {
        RewindFuture::new(self, marker)
    }
}
