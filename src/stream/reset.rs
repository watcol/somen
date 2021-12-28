use core::pin::Pin;
use core::task::{Context, Poll};

mod future;
use future::{MarkFuture, ResetFuture};

pub trait Reset {
    type Marker;
    fn poll_mark(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Marker>;
    fn poll_reset(self: Pin<&mut Self>, cx: &mut Context<'_>, marker: Self::Marker) -> Poll<()>;

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
