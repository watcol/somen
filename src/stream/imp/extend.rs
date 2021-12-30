use super::{Positioned, Rewind};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, Stream, TryStream};
use pin_project_lite::pin_project;

pin_project! {
    /// Wrapping [`TryStream`], storing the stream outputs to any types implementing [`Extend`].
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    /// [`Extend`]: core::iter::Extend
    #[derive(Debug)]
    pub struct ExtendStream<'a, S: TryStream, E: ?Sized> {
        #[pin]
        stream: S,
        output: &'a mut E,
    }
}

impl<'a, S: TryStream, E: ?Sized> ExtendStream<'a, S, E> {
    /// Creating a new instance.
    #[inline]
    pub fn new(stream: S, output: &'a mut E) -> Self {
        Self { stream, output }
    }

    /// Extracting the original stream.
    #[inline]
    pub fn into_inner(self) -> S {
        self.stream
    }
}

impl<S: TryStream, E: Extend<S::Ok> + ?Sized> Stream for ExtendStream<'_, S, E>
where
    S::Ok: Clone,
{
    type Item = Result<S::Ok, S::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let res = ready!(this.stream.try_poll_next(cx));
        if let Some(Ok(ref i)) = res {
            this.output.extend(Some(i.clone()));
        }
        Poll::Ready(res)
    }
}

impl<S: Positioned, E: Extend<S::Ok> + ?Sized> Positioned for ExtendStream<'_, S, E>
where
    S::Ok: Clone,
{
    type Position = S::Position;

    #[inline]
    fn poll_position(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Position, Self::Error>> {
        self.project().stream.poll_position(cx)
    }
}

impl<S: Rewind, E: Extend<S::Ok> + ?Sized> Rewind for ExtendStream<'_, S, E>
where
    S::Ok: Clone,
{
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
