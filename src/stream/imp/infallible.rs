use core::convert::Infallible;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{FusedStream, Stream};
use pin_project_lite::pin_project;

pin_project! {
    /// Wraps normal(infallible) [`Stream`], implements [`TryStream`].
    ///
    /// The [`TryStream`] always returns [`Ok`], and the error type is [`Infallible`].
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    #[derive(Debug)]
    pub struct InfallibleStream<S> {
        #[pin]
        inner: S,
    }
}

impl<S: Stream> From<S> for InfallibleStream<S> {
    #[inline]
    fn from(stream: S) -> Self {
        Self { inner: stream }
    }
}

impl<S: Stream> InfallibleStream<S> {
    /// Creates a new instance.
    #[inline]
    pub fn new(stream: S) -> Self {
        Self::from(stream)
    }

    /// Extracts the original stream.
    #[inline]
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S: FusedStream> FusedStream for InfallibleStream<S> {
    #[inline]
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<S: Stream> Stream for InfallibleStream<S> {
    type Item = Result<S::Item, Infallible>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx).map(|i| i.map(Ok))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
