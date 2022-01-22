use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::Stream;
use pin_project_lite::pin_project;

pin_project! {
    /// Wrapping [`Iterator`], implements [`Stream`].
    ///
    /// ### Note
    /// The returned stream is not a [`TryStream`], you should combinate it with
    /// [`InfallibleStream`]. An utility function [`from_iter`] will automatically
    /// do this.
    ///
    /// [`TryStream`]: futures_core::TryStream
    /// [`InfallibleStream`]: crate::stream::InfallibleStream
    /// [`from_iter`]: crate::stream::from_iter
    #[derive(Debug)]
    pub struct IteratorStream<I> {
        iter: I,
    }
}

impl<I: Iterator> From<I> for IteratorStream<I> {
    #[inline]
    fn from(iter: I) -> Self {
        Self { iter }
    }
}

impl<I: Iterator> IteratorStream<I> {
    /// Create a new instance.
    #[inline]
    pub fn new(iter: I) -> Self {
        Self::from(iter)
    }

    /// Extracting the original iterator.
    #[inline]
    pub fn into_inner(self) -> I {
        self.iter
    }
}

impl<I: Iterator> Stream for IteratorStream<I> {
    type Item = I::Item;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.project().iter.next())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
