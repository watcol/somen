mod error;
pub use error::SeekError;

use crate::stream::{Positioned, Rewind};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{Stream, TryStream};
use futures_io::{AsyncSeek, SeekFrom};
use pin_project_lite::pin_project;

pin_project! {
    /// Wrapping [`AsyncSeek`], implements [`Positioned`] and [`Rewind`] trait.
    ///
    /// [`AsyncSeek`]: futures_io::AsyncSeek
    /// [`Positioned`]: crate::stream::position::Positioned
    /// [`Rewind`]: crate::stream::rewind::Rewind
    #[derive(Debug)]
    #[cfg_attr(all(doc, feature = "unstable"), doc(cfg(feature = "std")))]
    pub struct SeekRewinder<S> {
        #[pin]
        stream: S,
    }
}

impl<S: TryStream + AsyncSeek> From<S> for SeekRewinder<S> {
    #[inline]
    fn from(stream: S) -> Self {
        Self { stream }
    }
}

impl<S: TryStream + AsyncSeek> SeekRewinder<S> {
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

impl<S: TryStream> Stream for SeekRewinder<S> {
    type Item = Result<S::Ok, SeekError<S::Error>>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project()
            .stream
            .try_poll_next(cx)
            .map(|o| o.map(|r| r.map_err(SeekError::Stream)))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.stream.size_hint().0, None)
    }
}

impl<S: TryStream + AsyncSeek> Positioned for SeekRewinder<S> {
    type Position = u64;

    #[inline]
    fn poll_position(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Position, Self::Error>> {
        self.project()
            .stream
            .poll_seek(cx, SeekFrom::Current(0))
            .map(|r| r.map_err(SeekError::Seek))
    }
}

impl<S: TryStream + AsyncSeek> Rewind for SeekRewinder<S> {
    type Marker = u64;

    #[inline]
    fn poll_mark(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Marker, Self::Error>> {
        self.poll_position(cx)
    }

    #[inline]
    fn poll_rewind(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        marker: Self::Marker,
    ) -> Poll<Result<(), Self::Error>> {
        self.project()
            .stream
            .poll_seek(cx, SeekFrom::Start(marker))
            .map(|r| r.map(|_| ()).map_err(SeekError::Seek))
    }
}
