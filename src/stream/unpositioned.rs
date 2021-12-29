use super::{Positioned, Unpositioned};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{Stream, TryStream};
use pin_project_lite::pin_project;

pin_project! {
    /// Wrapping a [`TryStream`], and just implementing [`Unpositioned`] trait.
    ///
    /// [`TryStream`]: https://docs.rs/futures/latest/futures/stream/trait.TryStream.html
    /// [`Unpositioned`]: ./trait.Unpositioned.html
    #[derive(Debug)]
    pub struct UnpositionedStream<S> {
        #[pin]
        stream: S,
    }
}

impl<S: TryStream> UnpositionedStream<S> {
    /// Creating a new instance.
    #[inline]
    pub fn new(stream: S) -> Self {
        Self { stream }
    }
}

impl<S: TryStream> Stream for UnpositionedStream<S> {
    type Item = Result<S::Ok, S::Error>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.try_poll_next(cx)
    }
}
