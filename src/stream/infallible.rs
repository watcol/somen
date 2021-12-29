use core::convert::Infallible;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::Stream;
use pin_project_lite::pin_project;

use super::Unpositioned;

pin_project! {
    /// Wrapping normal (infallible) [`Stream`] and implements [`TryStream`] which returns
    /// `Result<S::Item, core::convert::Infallible>` and [`Unpositioned`].
    ///
    /// [`Stream`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
    /// [`TryStream`]: https://docs.rs/futures/latest/futures/stream/trait.TryStream.html
    /// [`Unpositioned`]: ./trait.Unpositioned.html
    #[derive(Debug)]
    pub struct InfallibleStream<S> {
        #[pin]
        stream: S,
    }
}

impl<S: Stream> InfallibleStream<S> {
    /// Creating a new instance.
    #[inline]
    pub fn new(stream: S) -> Self {
        Self { stream }
    }
}

impl<S: Stream> Stream for InfallibleStream<S> {
    type Item = Result<S::Item, Infallible>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx).map(|i| i.map(Ok))
    }
}

impl<S: Stream> Unpositioned for InfallibleStream<S> {}
