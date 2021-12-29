mod future;
use future::PositionFuture;

use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::TryStream;

/// A stream that records its position.
///
/// if a stream does not have any information for its position, it should implement this by
/// `type Position = ();`.
///
/// In other words, this trait will be used in so many situations (e.g. returning the position
/// where an error has occured), all streams should implement this.
pub trait Positioned: TryStream {
    /// The type of the position.
    type Position: Clone + PartialOrd;

    /// Getting the current position.
    fn poll_position(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Position, Self::Error>>;

    /// An asynchronous version of `poll_position`, which returns a `Future` object.
    #[inline]
    fn position(&mut self) -> PositionFuture<'_, Self>
    where
        Self: Unpin,
    {
        PositionFuture::new(self)
    }
}
