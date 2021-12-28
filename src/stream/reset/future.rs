use super::Reset;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct MarkFuture<'a, P: ?Sized>(&'a mut P);

impl<'a, P: ?Sized> MarkFuture<'a, P> {
    #[inline]
    pub(super) fn new(inner: &'a mut P) -> Self {
        Self(inner)
    }
}

impl<P: ?Sized + Unpin> Unpin for MarkFuture<'_, P> {}

impl<'a, P: Reset + ?Sized + Unpin> Future for MarkFuture<'a, P> {
    type Output = Result<P::Marker, P::Error>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.0).poll_mark(cx)
    }
}

pub struct ResetFuture<'a, P: Reset + ?Sized>(&'a mut P, P::Marker);

impl<'a, P: Reset + ?Sized> ResetFuture<'a, P> {
    #[inline]
    pub(super) fn new(inner: &'a mut P, marker: P::Marker) -> Self {
        Self(inner, marker)
    }
}

impl<P: Reset + ?Sized + Unpin> Unpin for ResetFuture<'_, P> {}

impl<'a, P: Reset + ?Sized + Unpin> Future for ResetFuture<'a, P>
where
    P::Marker: Clone,
{
    type Output = Result<(), P::Error>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let marker = self.1.clone();
        Pin::new(&mut *self.0).poll_reset(cx, marker)
    }
}
