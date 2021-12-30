use core::convert::Infallible;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::Stream;
use pin_project_lite::pin_project;

pin_project! {
    /// Wrapping normal(infallible) [`Stream`], implements [`TryStream`].
    ///
    /// The [`TryStream`] always returns [`Ok`], and the error type is [`Infallible`].
    ///
    /// [`Stream`]: futures_core::stream::Stream
    /// [`TryStream`]: futures_core::stream::TryStream
    /// [`Ok`]: core::result::Result::Ok
    /// [`Infallible`]: core::convert::Infallible
    #[derive(Debug)]
    pub struct InfallibleStream<S> {
        #[pin]
        stream: S,
    }
}

impl<S: Stream> From<S> for InfallibleStream<S> {
    #[inline]
    fn from(stream: S) -> Self {
        Self { stream }
    }
}

impl<S: Stream> InfallibleStream<S> {
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

impl<S: Stream> Stream for InfallibleStream<S> {
    type Item = Result<S::Item, Infallible>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx).map(|i| i.map(Ok))
    }
}
