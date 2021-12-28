mod future;

use future::PositionFuture;

use core::pin::Pin;
use core::task::{Context, Poll};

pub trait Positioned {
    type Position: Clone;
    fn poll_position(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Position>;

    #[inline]
    fn position(&mut self) -> PositionFuture<'_, Self>
    where
        Self: Unpin,
    {
        PositionFuture::new(self)
    }
}
