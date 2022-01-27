#[cfg(feature = "alloc")]
use alloc::borrow::Cow;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, FusedStream, Stream, TryStream};
use pin_project_lite::pin_project;

use super::Locator;
#[cfg(feature = "alloc")]
use crate::stream::record::Record;
use crate::stream::{Positioned, Rewind};

pin_project! {
    /// Wrapping [`TryStream`], implements [`Positioned`] trait.
    #[derive(Debug)]
    pub struct PositionedStream<S, L> {
        #[pin]
        inner: S,
        position: L,
    }
}

impl<S, L: Default> From<S> for PositionedStream<S, L> {
    #[inline]
    fn from(inner: S) -> Self {
        Self {
            inner,
            position: L::default(),
        }
    }
}

impl<S, L> PositionedStream<S, L> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: S, position: L) -> Self {
        Self { inner, position }
    }

    /// Extracting the original stream.
    #[inline]
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S: TryStream + FusedStream, L: Locator<S::Ok>> FusedStream for PositionedStream<S, L> {
    #[inline]
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<S: TryStream, L: Locator<S::Ok>> Stream for PositionedStream<S, L> {
    type Item = Result<S::Ok, S::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let res = ready!(this.inner.try_poll_next(cx));
        if let Some(Ok(ref c)) = res {
            this.position.next(c);
        }
        Poll::Ready(res)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
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
    fn mark(self: Pin<&mut Self>) -> Result<Self::Marker, Self::Error> {
        self.project().inner.mark()
    }

    #[inline]
    fn rewind(self: Pin<&mut Self>, marker: Self::Marker) -> Result<(), Self::Error> {
        self.project().inner.rewind(marker)
    }

    #[inline]
    fn drop_marker(self: Pin<&mut Self>, marker: Self::Marker) -> Result<(), Self::Error> {
        self.project().inner.drop_marker(marker)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<S: Record + TryStream, L: Locator<S::Ok>> Record for PositionedStream<S, L> {
    type Borrowed = S::Borrowed;

    #[inline]
    fn start(self: Pin<&mut Self>) {
        self.project().inner.start()
    }

    fn end(self: Pin<&mut Self>) -> Cow<'_, Self::Borrowed> {
        self.project().inner.end()
    }
}
