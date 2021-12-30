use crate::stream::{Positioned, Rewind};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{Stream, TryStream};
use pin_project_lite::pin_project;

pin_project! {
    /// Wrapping [`TryStream`], just implements [`Positioned`] trait by `type Position = ()`.
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    /// [`Unpositioned`]: crate::stream::Unpositioned
    #[derive(Debug)]
    pub struct UnpositionedStream<S> {
        #[pin]
        stream: S,
    }
}

impl<S: TryStream> From<S> for UnpositionedStream<S> {
    #[inline]
    fn from(stream: S) -> Self {
        Self { stream }
    }
}

impl<S: TryStream> UnpositionedStream<S> {
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

impl<S: TryStream> Stream for UnpositionedStream<S> {
    type Item = Result<S::Ok, S::Error>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.try_poll_next(cx)
    }
}

impl<S: TryStream> Positioned for UnpositionedStream<S> {
    type Position = ();

    #[inline]
    fn poll_position(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Position, Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl<S: Rewind> Rewind for UnpositionedStream<S> {
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
