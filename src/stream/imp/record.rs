use super::{Positioned, Rewind};
use alloc::vec::Vec;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, Stream, TryStream};
use pin_project_lite::pin_project;

pin_project! {
    /// Wrapping [`TryStream`],  implements [`Positioned`] and [`Rewind`] trait by storing
    /// the stream outputs to [`Vec`].
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    /// [`Positioned`]: crate::stream::Positioned
    /// [`Rewind`]: crate::stream::Rewind
    /// [`Vec`]: alloc::vec::Vec
    #[derive(Debug)]
    pub struct RecordStream<S: TryStream> {
        position: usize,
        record: Vec<S::Ok>,
        #[pin]
        stream: S,
    }
}

impl<S: TryStream> RecordStream<S> {
    /// Creating a new instance.
    #[inline]
    pub fn new(stream: S) -> Self {
        Self {
            position: 0,
            record: Vec::new(),
            stream,
        }
    }

    /// Getting the reference of the vector.
    pub fn as_vec(&self) -> &Vec<S::Ok> {
        &self.record
    }

    /// Getting the mutable reference of the vector.
    pub fn as_vec_mut(&mut self) -> &mut Vec<S::Ok> {
        &mut self.record
    }

    /// Extracting the vector from the stream.
    pub fn into_vec(self) -> Vec<S::Ok> {
        self.record
    }
}

impl<S: TryStream> Stream for RecordStream<S>
where
    S::Ok: Clone,
{
    type Item = Result<S::Ok, S::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if *this.position == this.record.len() {
            let res = ready!(this.stream.try_poll_next(cx));
            if let Some(Ok(ref i)) = res {
                *this.position += 1;
                this.record.push(i.clone());
            }
            Poll::Ready(res)
        } else {
            let res = this.record.get(*this.position).cloned().map(Ok);
            *this.position += 1;
            Poll::Ready(res)
        }
    }
}

impl<S: TryStream> Positioned for RecordStream<S>
where
    S::Ok: Clone,
{
    type Position = usize;

    #[inline]
    fn poll_position(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Position, Self::Error>> {
        Poll::Ready(Ok(*self.project().position))
    }
}

impl<S: TryStream> Rewind for RecordStream<S>
where
    S::Ok: Clone,
{
    type Marker = usize;

    #[inline]
    fn poll_mark(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Marker, Self::Error>> {
        self.poll_position(cx)
    }

    fn poll_rewind(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        marker: Self::Marker,
    ) -> Poll<Result<(), Self::Error>> {
        *self.project().position = marker;
        Poll::Ready(Ok(()))
    }
}
