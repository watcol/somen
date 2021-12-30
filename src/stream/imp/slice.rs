use crate::stream::{Positioned, Rewind};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::Stream;
use pin_project_lite::pin_project;
use std::convert::Infallible;

pin_project! {
    /// Wrapping slices, implements [`TryStream`], [`Positioned`] and [`Rewind`] trait.
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    /// [`Positioned`]: crate::stream::position::Positioned
    /// [`Rewind`]: crate::stream::rewind::Rewind
    #[derive(Debug)]
    pub struct SliceStream<'a, T> {
        position: usize,
        slice: &'a [T],
    }
}

impl<'a, T: Clone> From<&'a [T]> for SliceStream<'a, T> {
    #[inline]
    fn from(slice: &'a [T]) -> Self {
        Self { position: 0, slice }
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
}

impl<T: Clone> Positioned for SliceStream<'_, T> {
    type Position = usize;

    #[inline]
    fn poll_position(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Position, Self::Error>> {
        Poll::Ready(Ok(*self.project().position))
    }
}

impl<T: Clone> Rewind for SliceStream<'_, T> {
    type Marker = usize;

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
        _cx: &mut Context<'_>,
        marker: Self::Marker,
    ) -> Poll<Result<(), Self::Error>> {
        *self.project().position = marker;
        Poll::Ready(Ok(()))
    }
}
