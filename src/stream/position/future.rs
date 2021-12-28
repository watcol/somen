use super::Positioned;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct PositionFuture<'a, P: ?Sized>(&'a mut P);

impl<P: ?Sized + Unpin> Unpin for PositionFuture<'_, P> {}

impl<'a, P: ?Sized> PositionFuture<'a, P> {
    #[inline]
    pub(super) fn new(inner: &'a mut P) -> Self {
        Self(inner)
    }
}

impl<P: Positioned + ?Sized + Unpin> Future for PositionFuture<'_, P> {
    type Output = Result<P::Position, P::Error>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.0).poll_position(cx)
    }
}
