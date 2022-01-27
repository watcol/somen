#[cfg(feature = "alloc")]
use alloc::borrow::Cow;
use core::convert::Infallible;
#[cfg(feature = "alloc")]
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::Stream;
use pin_project_lite::pin_project;

#[cfg(feature = "alloc")]
use crate::stream::record::Record;
use crate::stream::{Positioned, Rewind};

pin_project! {
    /// Wrapping slices, implements [`TryStream`], [`Positioned`], [`Rewind`] and [`Record`] trait.
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    #[derive(Debug)]
    pub struct SliceStream<'a, T> {
        slice: &'a [T],
        position: usize,
        recording_pos: Option<usize>,
    }
}

impl<'a, T: Clone> From<&'a [T]> for SliceStream<'a, T> {
    #[inline]
    fn from(slice: &'a [T]) -> Self {
        Self {
            slice,
            position: 0,
            recording_pos: None,
        }
    }
}
impl<'a, T: Clone> SliceStream<'a, T> {
    /// Creating a new instance.
    #[inline]
    pub fn new(slice: &'a [T]) -> Self {
        Self::from(slice)
    }
}

impl<T: Clone> Stream for SliceStream<'_, T> {
    type Item = Result<T, Infallible>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let res = this.slice.get(*this.position);
        *this.position += 1;
        Poll::Ready(res.cloned().map(Ok))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.slice.len(), Some(self.slice.len()))
    }
}

impl<T: Clone> Positioned for SliceStream<'_, T> {
    type Locator = usize;

    #[inline]
    fn position(&self) -> Self::Locator {
        self.position
    }
}

impl<T: Clone> Rewind for SliceStream<'_, T> {
    type Marker = usize;

    #[inline]
    fn mark(self: Pin<&mut Self>) -> Result<Self::Marker, Self::Error> {
        Ok(self.position())
    }

    #[inline]
    fn rewind(mut self: Pin<&mut Self>, marker: Self::Marker) -> Result<(), Self::Error> {
        self.position = marker;
        Ok(())
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<T: Clone> Record for SliceStream<'_, T> {
    type Borrowed = [T];

    fn start(self: Pin<&mut Self>) {
        let this = self.project();
        *this.recording_pos = Some(*this.position);
    }

    fn end(self: Pin<&mut Self>) -> Cow<'_, Self::Borrowed> {
        let this = self.project();
        let pos = mem::take(this.recording_pos).unwrap_or(*this.position);
        Cow::from(
            this.slice
                .get(pos..*this.position)
                .expect("recording_pos <= position"),
        )
    }
}
