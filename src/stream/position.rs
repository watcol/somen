mod future;
use future::PositionFuture;

use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::TryStream;

pub trait Positioned: TryStream {
    type Position: Clone;
    fn poll_position(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Position, Self::Error>>;

    #[inline]
    fn position(&mut self) -> PositionFuture<'_, Self>
    where
        Self: Unpin,
    {
        PositionFuture::new(self)
    }
}
