mod future;
use future::PositionFuture;

use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::TryStream;

/// A stream that records its position.
///
/// if a stream does not have any information for its position, it should implement this by
/// implementing `Unpositioned` trait.
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

    /// An asynchronous version of [`poll_position`](#tymethod.poll_position), which returns a
    /// [`Future`](https://doc.rust-lang.org/stable/std/future/trait.Future.html) object.
    #[inline]
    fn position(&mut self) -> PositionFuture<'_, Self>
    where
        Self: Unpin,
    {
        PositionFuture::new(self)
    }
}

/// A stream does not records its position.
///
/// By implementing this trait, the type automatically implements `Positioned` trait by `type
/// Position = ();`.
pub trait Unpositioned: TryStream {}

impl<T: Unpositioned> Positioned for T {
    type Position = ();

    #[inline]
    fn poll_position(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Position, Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
