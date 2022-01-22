use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, FusedStream, Stream, TryStream};
use pin_project_lite::pin_project;

use crate::stream::Positioned;

pin_project! {
    /// Wrapping [`TryStream`], storing the stream outputs to any types implementing [`Extend`].
    #[derive(Debug)]
    pub struct ExtendRecorder<'a, S: TryStream, E: ?Sized> {
        #[pin]
        inner: S,
        output: &'a mut E,
    }
}

impl<'a, S: TryStream, E: ?Sized> ExtendRecorder<'a, S, E> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: S, output: &'a mut E) -> Self {
        Self { inner, output }
    }

    /// Extracting the original stream.
    #[inline]
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S: TryStream + FusedStream, E: Extend<S::Ok> + ?Sized> FusedStream for ExtendRecorder<'_, S, E>
where
    S::Ok: Clone,
{
    #[inline]
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<S: TryStream, E: Extend<S::Ok> + ?Sized> Stream for ExtendRecorder<'_, S, E>
where
    S::Ok: Clone,
{
    type Item = Result<S::Ok, S::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let res = ready!(this.inner.try_poll_next(cx));
        if let Some(Ok(ref i)) = res {
            this.output.extend(Some(i.clone()));
        }
        Poll::Ready(res)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S: Positioned, E: Extend<S::Ok> + ?Sized> Positioned for ExtendRecorder<'_, S, E>
where
    S::Ok: Clone,
{
    type Locator = S::Locator;

    #[inline]
    fn position(&self) -> Self::Locator {
        self.inner.position()
    }
}
