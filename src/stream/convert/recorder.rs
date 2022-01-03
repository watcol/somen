use pin_project_lite::pin_project;

use crate::stream::position::Positioned;

use super::{ConvertError, Converter};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, Stream, TryStream};

pin_project! {
    /// Wrapping a [`TryStream`], recording items converted by [`Converter`] using [`Extend`].
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    /// [`Converter`]: crate::stream::convert::Converter
    /// [`Extend`]: core::iter::Extend
    #[derive(Debug)]
    pub struct ConvertedRecorder<'a, S, E, C> {
        output: &'a mut E,
        #[pin]
        stream: S,
        converter: C,
    }
}

impl<S, E, C> Stream for ConvertedRecorder<'_, S, E, C>
where
    S: TryStream,
    S::Ok: Clone,
    E: Extend<C::Output>,
    C: Converter<S::Ok>,
{
    type Item = Result<S::Ok, ConvertError<S::Error, C::Error>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let res = match ready!(this.stream.try_poll_next(cx)) {
            Some(Ok(item)) => match this.converter.convert(item.clone(), *this.output) {
                Ok(_) => Some(Ok(item)),
                Err(e) => Some(Err(ConvertError::Conversion(e))),
            },
            Some(Err(e)) => Some(Err(ConvertError::Stream(e))),
            None => None,
        };
        Poll::Ready(res)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

impl<S, E, C> Positioned for ConvertedRecorder<'_, S, E, C>
where
    S: Positioned,
    S::Ok: Clone,
    E: Extend<C::Output>,
    C: Converter<S::Ok>,
{
    type Position = S::Position;

    #[inline]
    fn poll_position(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Position, Self::Error>> {
        self.project()
            .stream
            .poll_position(cx)
            .map_err(ConvertError::Stream)
    }
}
