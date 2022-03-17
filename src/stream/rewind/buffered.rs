mod error;

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, FusedStream, Stream, TryStream};
use pin_project_lite::pin_project;

use crate::stream::{Positioned, Rewind};
pub use error::BufferedError;

pin_project! {
    /// Wraps [`TryStream`],  implements [`Positioned`] and [`Rewind`] trait by storing
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
        recording_pos: Option<usize>,
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
            recording_pos: None,
        }
    }
}

impl<S: TryStream> BufferedRewinder<S> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: S) -> Self {
        Self::from(inner)
    }

    /// Extracts the original stream.
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
                if !this.markers.is_empty() || this.recording_pos.is_some() {
                    this.buffer.push_back(i.clone());
                } else {
                    *this.buffer_offset += 1;
                }
            }
            Poll::Ready(res)
        } else if *this.position == *this.buffer_offset
            && this.markers.is_empty()
            && this.recording_pos.is_none()
        {
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

    fn mark(self: Pin<&mut Self>) -> Result<Self::Marker, Self::Error> {
        let this = self.project();
        this.markers.push(*this.position);
        Ok(*this.position)
    }

    fn rewind(self: Pin<&mut Self>, marker: Self::Marker) -> Result<(), Self::Error> {
        let this = self.project();
        if this.markers.pop() == Some(marker) {
            *this.position = marker;
            Ok(())
        } else {
            Err(BufferedError::Buffer)
        }
    }

    fn drop_marker(self: Pin<&mut Self>, marker: Self::Marker) -> Result<(), Self::Error> {
        if self.project().markers.pop() == Some(marker) {
            Ok(())
        } else {
            Err(BufferedError::Buffer)
        }
    }
}
