mod error;

pub use error::BufferedError;

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, FusedStream, Stream, TryStream};
use pin_project_lite::pin_project;

use crate::stream::{Positioned, Rewind};

pin_project! {
    /// Wrapping [`TryStream`],  implements [`Positioned`] and [`Rewind`] trait by storing
    /// recent output to buffer, which will live until it becomes unneeded.
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    /// [`Positioned`]: crate::stream::position::Positioned
    /// [`Rewind`]: crate::stream::positon::Rewind
    #[derive(Debug)]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    pub struct BufferedRewinder<S: TryStream> {
        #[pin]
        inner: S,
        position: usize,
        buffer: VecDeque<S::Ok>,
        buffer_offset: usize,
        markers: Vec<usize>,
    }
}

impl<S: TryStream> From<S> for BufferedRewinder<S> {
    #[inline]
    fn from(inner: S) -> Self {
        Self {
            inner,
            position: 0,
            buffer: VecDeque::new(),
            buffer_offset: 0,
            markers: Vec::new(),
        }
    }
}

impl<S: TryStream> BufferedRewinder<S> {
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

impl<S: TryStream + FusedStream> FusedStream for BufferedRewinder<S>
where
    S::Ok: Clone,
{
    #[inline]
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<S: TryStream> Stream for BufferedRewinder<S>
where
    S::Ok: Clone,
{
    type Item = Result<S::Ok, BufferedError<S::Error>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if *this.position == *this.buffer_offset + this.buffer.len() {
            let res =
                ready!(this.inner.try_poll_next(cx)).map(|r| r.map_err(BufferedError::Stream));
            if let Some(Ok(ref i)) = res {
                *this.position += 1;
                if !this.markers.is_empty() {
                    this.buffer.push_back(i.clone());
                }
            }
            Poll::Ready(res)
        } else if *this.position == *this.buffer_offset && this.markers.is_empty() {
            let res = this.buffer.pop_front();
            *this.position += 1;
            *this.buffer_offset += 1;
            Poll::Ready(res.map(Ok))
        } else {
            let res = this.buffer.get(*this.position - *this.buffer_offset);
            *this.position += 1;
            Poll::Ready(res.cloned().map(Ok))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.inner.size_hint().0, None)
    }
}

impl<S: TryStream> Positioned for BufferedRewinder<S>
where
    S::Ok: Clone,
{
    type Locator = usize;

    #[inline]
    fn position(&self) -> Self::Locator {
        self.position
    }
}

impl<S: TryStream> Rewind for BufferedRewinder<S>
where
    S::Ok: Clone,
{
    type Marker = usize;

    fn poll_mark(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Marker, Self::Error>> {
        let this = self.project();
        if this.markers.is_empty() {
            *this.buffer_offset = *this.position;
        }
        this.markers.push(*this.position);
        Poll::Ready(Ok(*this.position))
    }

    fn poll_rewind(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        marker: Self::Marker,
    ) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        if this.markers.pop() == Some(marker) {
            *this.position = marker;
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(BufferedError::Buffer))
        }
    }
}
