use pin_project_lite::pin_project;

use super::{ConvertError, Converter};
use alloc::collections::VecDeque;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, Stream, TryStream};

pin_project! {
    /// Wrapping a [`TryStream`], converting its items using [`Converter`].
    ///
    /// [`TryStream`]: futures_core::stream::TryStream
    /// [`Converter`]: crate::stream::convert::Converter
    #[derive(Debug)]
    #[cfg_attr(all(doc, feature = "unstable"), doc(cfg(feature = "alloc")))]
    pub struct ConvertedStream<S: TryStream, C: Converter<S::Ok>> {
        buffer: VecDeque<C::Output>,
        #[pin]
        stream: S,
        converter: C,
    }
}

impl<S: TryStream, C: Converter<S::Ok>> Stream for ConvertedStream<S, C> {
    type Item = Result<C::Output, ConvertError<S::Error, C::Error>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if !this.buffer.is_empty() {
            Poll::Ready(this.buffer.pop_front().map(Ok))
        } else {
            match ready!(this.stream.try_poll_next(cx)) {
                Some(Ok(item)) => match this.converter.convert(item, this.buffer) {
                    Ok(c) if c == 0 => Poll::Pending,
                    Ok(_) => Poll::Ready(this.buffer.pop_front().map(Ok)),
                    Err(e) => Poll::Ready(Some(Err(ConvertError::Conversion(e)))),
                },
                Some(Err(e)) => Poll::Ready(Some(Err(ConvertError::Stream(e)))),
                None => Poll::Ready(None),
            }
        }
    }
}
