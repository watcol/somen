use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, Stream};
use futures_io::{AsyncRead, AsyncSeek, Error};
use pin_project_lite::pin_project;

pin_project! {
    /// Wraps [`AsyncRead`], implements [`TryStream`] trait.
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
    pub struct ReaderStream<R> {
        #[pin]
        reader: R,
    }
}

impl<R: AsyncRead> From<R> for ReaderStream<R> {
    #[inline]
    fn from(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: AsyncRead> ReaderStream<R> {
    /// Creates a new instance.
    #[inline]
    pub fn new(reader: R) -> Self {
        Self::from(reader)
    }

    /// Extracts the original reader.
    #[inline]
    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<R: AsyncRead> Stream for ReaderStream<R> {
    type Item = Result<u8, Error>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = [0u8; 1];
        Poll::Ready(
            match ready!(self.project().reader.poll_read(cx, &mut buf)) {
                Ok(0) => None,
                Ok(_) => Some(Ok(buf[0])),
                Err(e) => Some(Err(e)),
            },
        )
    }
}

impl<R: AsyncSeek> AsyncSeek for ReaderStream<R> {
    fn poll_seek(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        pos: futures_io::SeekFrom,
    ) -> Poll<futures_io::Result<u64>> {
        self.project().reader.poll_seek(cx, pos)
    }
}
