use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub trait Positioned {
    type Position: Clone;
    fn poll_position(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Position>;

    #[inline]
    fn position(&mut self) -> PositionFuture<'_, Self> {
        PositionFuture(self)
    }
}

pub struct PositionFuture<'a, P: ?Sized>(&'a mut P);

impl<P: ?Sized + Unpin> Unpin for PositionFuture<'_, P> {}

impl<P: Positioned + ?Sized + Unpin> Future for PositionFuture<'_, P> {
    type Output = P::Position;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.0).poll_position(cx)
    }
}
