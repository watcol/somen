use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::Stream;
use pin_project_lite::pin_project;

use super::Unpositioned;

pin_project! {
    /// Wrapping [`Iterator`], implements [`Stream`].
    ///
    /// if `I::Item == Result<T, E>`, this implements [`Unpositioned`], otherwise you should
    /// combinate with [`InfallibleStream`].
    ///
    /// [`Iterator`]: https://doc.rust-lang.org/stable/core/iter/trait.Iterator.html
    /// [`Stream`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
    /// [`Unpositioned`]: ./trait.Unpositioned.html
    /// [`InfallibleStream`]: ./struct.InfallibleStream.html
    #[derive(Debug)]
    pub struct IteratorStream<I> {
        iter: I,
    }
}

impl<I: Iterator> IteratorStream<I> {
    /// Create a new instance.
    #[inline]
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I: Iterator> Stream for IteratorStream<I> {
    type Item = I::Item;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.project().iter.next())
    }
}

impl<I, T, E> Unpositioned for IteratorStream<I> where I: Iterator<Item = Result<T, E>> {}
