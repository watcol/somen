use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, FusedStream, Stream, TryStream};
use pin_project_lite::pin_project;

use super::Record;
use crate::stream::{Positioned, Rewind};

pin_project! {
    /// Wrapping [`TryStream`],  implements [`Positioned`] and [`Rewind`] trait by storing
    /// the stream outputs to [`Vec`].
    #[derive(Debug)]
    pub struct VecRecorder<S: TryStream> {
        #[pin]
        inner: S,
        position: usize,
        recording_pos: Option<usize>,
        record: Vec<S::Ok>,
    }
}

impl<S: TryStream> From<S> for VecRecorder<S> {
    #[inline]
    fn from(inner: S) -> Self {
        Self {
            inner,
            position: 0,
            recording_pos: None,
            record: Vec::new(),
        }
    }
}

impl<S: TryStream> VecRecorder<S> {
    /// Creating a new instance.
    #[inline]
    pub fn new(stream: S) -> Self {
        Self::from(stream)
    }

    /// Extracting the original stream.
    #[inline]
    pub fn into_inner(self) -> S {
        self.inner
    }

    /// Getting the reference of the vector.
    #[inline]
    pub fn as_vec(&self) -> &Vec<S::Ok> {
        &self.record
    }

    /// Getting the mutable reference of the vector.
    #[inline]
    pub fn as_vec_mut(&mut self) -> &mut Vec<S::Ok> {
        &mut self.record
    }

    /// Extracting the vector from the stream.
    #[inline]
    pub fn into_vec(self) -> Vec<S::Ok> {
        self.record
    }
}

impl<S: TryStream + FusedStream> FusedStream for VecRecorder<S>
where
    S::Ok: Clone,
{
    #[inline]
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<S: TryStream> Stream for VecRecorder<S>
where
    S::Ok: Clone,
{
    type Item = Result<S::Ok, S::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if *this.position == this.record.len() {
            let res = ready!(this.inner.try_poll_next(cx));
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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.inner.size_hint().0, None)
    }
}

impl<S: TryStream> Positioned for VecRecorder<S>
where
    S::Ok: Clone,
{
    type Locator = usize;

    #[inline]
    fn position(&self) -> Self::Locator {
        self.position
    }
}

impl<S: TryStream> Rewind for VecRecorder<S>
where
    S::Ok: Clone,
{
    type Marker = usize;

    #[inline]
    fn mark(self: Pin<&mut Self>) -> Result<Self::Marker, Self::Error> {
        Ok(self.position())
    }

    fn rewind(self: Pin<&mut Self>, marker: Self::Marker) -> Result<(), Self::Error> {
        *self.project().position = marker;
        Ok(())
    }
}

impl<S: TryStream> Record for VecRecorder<S>
where
    S::Ok: Clone,
{
    type Borrowed = [S::Ok];

    fn start(self: Pin<&mut Self>) {
        let this = self.project();
        *this.recording_pos = Some(*this.position);
    }

    fn end(self: Pin<&mut Self>) -> Option<Cow<'_, Self::Borrowed>> {
        let this = self.project();
        let pos = mem::take(this.recording_pos)?;
        this.record.get(pos..*this.position).map(Cow::from)
    }
}
