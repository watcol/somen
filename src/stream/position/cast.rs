mod error;
pub use error::CastError;

use crate::stream::{Positioned, Rewind};
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::Stream;
use pin_project_lite::pin_project;

pin_project! {
    /// Performing type casting on `S::Position` into `T` using [`TryFrom`].
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    /// [`TryFrom`]: core::convert::TryFrom
    #[derive(Debug)]
    pub struct CastPositioner<S, T> {
        #[pin]
        stream: S,
        _phantom: PhantomData<T>,
    }
}

impl<S: Positioned, T: TryFrom<S::Position>> From<S> for CastPositioner<S, T> {
    #[inline]
    fn from(stream: S) -> Self {
        Self {
            stream,
            _phantom: PhantomData,
        }
    }
}

impl<S: Positioned, T: TryFrom<S::Position>> CastPositioner<S, T> {
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

impl<S: Positioned, T: TryFrom<S::Position>> Stream for CastPositioner<S, T> {
    type Item = Result<S::Ok, CastError<S::Error, T::Error>>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project()
            .stream
            .try_poll_next(cx)
            .map(|p| p.map(|o| o.map_err(CastError::Stream)))
    }
}

impl<S: Positioned, T> Positioned for CastPositioner<S, T>
where
    T: TryFrom<S::Position>,
{
    type Position = T;

    fn poll_position(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Position, Self::Error>> {
        self.project().stream.poll_position(cx).map(|p| {
            p.map_err(CastError::Stream)
                .and_then(|p| T::try_from(p).map_err(CastError::Convert))
        })
    }
}

impl<S: Positioned + Rewind, T: TryFrom<S::Position>> Rewind for CastPositioner<S, T> {
    type Marker = S::Marker;

    #[inline]
    fn poll_mark(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Marker, Self::Error>> {
        self.project()
            .stream
            .poll_mark(cx)
            .map_err(CastError::Stream)
    }

    #[inline]
    fn poll_rewind(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        marker: Self::Marker,
    ) -> Poll<Result<(), Self::Error>> {
        self.project()
            .stream
            .poll_rewind(cx, marker)
            .map_err(CastError::Stream)
    }
}
