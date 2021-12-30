use super::Positioned;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{Stream, TryStream};
use pin_project_lite::pin_project;

pin_project! {
    /// Wrapping [`TryStream`], implements [`Positioned`] trait.
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    /// [`Positioned`]: crate::stream::Positioned
    #[derive(Debug)]
    pub struct PositionedStream<S> {
        position: usize,
        #[pin]
        stream: S,
    }
}

impl<S: TryStream> PositionedStream<S> {
    /// Creating a new instance.
    #[inline]
    pub fn new(stream: S) -> Self {
        Self {
            position: 0,
            stream,
        }
    }
}

impl<S: TryStream> Stream for PositionedStream<S> {
    type Item = Result<S::Ok, S::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        *this.position += 1;
        this.stream.try_poll_next(cx)
    }
}

impl<S: TryStream> Positioned for PositionedStream<S> {
    type Position = usize;

    fn poll_position(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Position, Self::Error>> {
        Poll::Ready(Ok(*self.project().position))
    }
}
