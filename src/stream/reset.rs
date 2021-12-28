mod future;
use future::{MarkFuture, ResetFuture};

use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::TryStream;

pub trait Reset: TryStream {
    type Marker;
    fn poll_mark(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Marker, Self::Error>>;
    fn poll_reset(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        marker: Self::Marker,
    ) -> Poll<Result<(), Self::Error>>;

    #[inline]
    fn mark(&mut self) -> MarkFuture<'_, Self>
    where
        Self: Unpin,
    {
        MarkFuture::new(self)
    }

    #[inline]
    fn reset(&mut self, marker: Self::Marker) -> ResetFuture<'_, Self>
    where
        Self: Unpin,
        Self::Marker: Clone,
    {
        ResetFuture::new(self, marker)
    }
}
