mod error;

pub use error::SeekError;

use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{FusedStream, Stream, TryStream};
use futures_io::{AsyncSeek, SeekFrom};
use pin_project_lite::pin_project;

use crate::stream::{Positioned, Rewind};

pin_project! {
    /// Wrapping [`AsyncSeek`], implements [`Positioned`] and [`Rewind`] trait.
    ///
    /// [`AsyncSeek`]: futures_io::AsyncSeek
    /// [`Positioned`]: crate::stream::position::Positioned
    /// [`Rewind`]: crate::stream::rewind::Rewind
    #[derive(Debug)]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
    pub struct SeekRewinder<S> {
        #[pin]
        inner: S,
    }
}

impl<S: TryStream + AsyncSeek> From<S> for SeekRewinder<S> {
    #[inline]
    fn from(inner: S) -> Self {
        Self { inner }
    }
}

impl<S: TryStream + AsyncSeek> SeekRewinder<S> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: S) -> Self {
        Self::from(inner)
    }

    /// Extracting the original stream.
    #[inline]
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S: TryStream + FusedStream> FusedStream for SeekRewinder<S> {
    #[inline]
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<S: TryStream> Stream for SeekRewinder<S> {
    type Item = Result<S::Ok, SeekError<S::Error>>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project()
            .inner
            .try_poll_next(cx)
            .map(|o| o.map(|r| r.map_err(SeekError::Stream)))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.inner.size_hint().0, None)
    }
}

impl<S: Positioned> Positioned for SeekRewinder<S> {
    type Locator = S::Locator;

    #[inline]
    fn position(&self) -> Self::Locator {
        self.inner.position()
    }
}

impl<S: TryStream + AsyncSeek> Rewind for SeekRewinder<S> {
    type Marker = u64;

    #[inline]
    fn poll_mark(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Marker, Self::Error>> {
        self.project()
            .inner
            .poll_seek(cx, SeekFrom::Current(0))
            .map(|r| r.map_err(SeekError::Seek))
    }

    #[inline]
    fn poll_rewind(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        marker: Self::Marker,
    ) -> Poll<Result<(), Self::Error>> {
        self.project()
            .inner
            .poll_seek(cx, SeekFrom::Start(marker))
            .map(|r| r.map(|_| ()).map_err(SeekError::Seek))
    }
}
