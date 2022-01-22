use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, Stream, TryStream};
use pin_project_lite::pin_project;

use super::Locator;
use crate::stream::{Positioned, Rewind};

pin_project! {
    /// Wrapping [`TryStream`], implements [`Positioned`] trait.
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    /// [`Positioned`]: crate::stream::position::Positioned
    #[derive(Debug)]
    pub struct PositionedStream<S, L> {
        position: L,
        #[pin]
        stream: S,
    }
}

impl<S, L: Default> From<S> for PositionedStream<S, L> {
    #[inline]
    fn from(stream: S) -> Self {
        Self {
            position: L::default(),
            stream,
        }
    }
}

impl<S, L> PositionedStream<S, L> {
    /// Creating a new instance.
    #[inline]
    pub fn new(stream: S, position: L) -> Self {
        Self { position, stream }
    }

    /// Extracting the original stream.
    #[inline]
    pub fn into_inner(self) -> S {
        self.stream
    }
}

impl<S: TryStream, L: Locator<S::Ok>> Stream for PositionedStream<S, L> {
    type Item = Result<S::Ok, S::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let res = ready!(this.stream.try_poll_next(cx));
        if let Some(Ok(ref c)) = res {
            this.position.next(c);
        }
        Poll::Ready(res)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

impl<S: TryStream, L: Locator<S::Ok> + Clone> Positioned for PositionedStream<S, L> {
    type Locator = L;

    #[inline]
    fn position(&self) -> Self::Locator {
        self.position.clone()
    }
}

impl<S: Rewind, L: Locator<S::Ok>> Rewind for PositionedStream<S, L> {
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
