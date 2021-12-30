use crate::stream::{Positioned, Rewind};
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
    pub struct Positioner<S> {
        position: usize,
        #[pin]
        stream: S,
    }
}

impl<S: TryStream> From<S> for Positioner<S> {
    #[inline]
    fn from(stream: S) -> Self {
        Self {
            position: 0,
            stream,
        }
    }
}
impl<S: TryStream> Positioner<S> {
    /// Creating a new instance.
    #[inline]
    pub fn new(stream: S) -> Self {
        Self::from(stream)
    }

    /// Extracting the original stream.
    #[inline]
    pub fn into_inner(self) -> S {
        self.stream
    }
}

impl<S: TryStream> Stream for Positioner<S> {
    type Item = Result<S::Ok, S::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        *this.position += 1;
        this.stream.try_poll_next(cx)
    }
}

impl<S: TryStream> Positioned for Positioner<S> {
    type Position = usize;

    #[inline]
    fn poll_position(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Position, Self::Error>> {
        Poll::Ready(Ok(*self.project().position))
    }
}

impl<S: Rewind> Rewind for Positioner<S> {
    type Marker = S::Marker;

    #[inline]
    fn poll_mark(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Marker, Self::Error>> {
        self.project().stream.poll_mark(cx)
    }

    #[inline]
    fn poll_rewind(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        marker: Self::Marker,
    ) -> Poll<Result<(), Self::Error>> {
        self.project().stream.poll_rewind(cx, marker)
    }
}
