use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;
use futures_core::stream::{Stream, TryStream};
use pin_project_lite::pin_project;

use super::BorrowMutPin;
use crate::error::{ParseError, PositionedError};
use crate::stream::position::Positioned;
use crate::stream::BasicInput;

pin_project! {
    /// A wrapping stream type for [`parser_stream_positioned`].
    ///
    /// [`parser_stream_positioned`]: self::StreamedParser::parser_stream_positioned
    #[derive(Debug)]
    pub struct PositionedStream<S, I: ?Sized> {
        #[pin]
        inner : S,
        _phantom: PhantomData<I>,
    }
}

impl<S, I: ?Sized> From<S> for PositionedStream<S, I> {
    #[inline]
    fn from(inner: S) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<S, I: ?Sized> PositionedStream<S, I> {
    /// Extracting the inner stream.
    #[inline]
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S, I, E, F> Stream for PositionedStream<S, I>
where
    S: TryStream<Error = ParseError<E, F>> + BorrowMutPin<I>,
    I: BasicInput<Error = F> + Positioned + ?Sized,
{
    #[allow(clippy::type_complexity)]
    type Item = Result<S::Ok, ParseError<PositionedError<E, I::Position>, F>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let start = match ready!(this.inner.as_mut().borrow_mut_pin().poll_position(cx)) {
            Ok(s) => s,
            Err(e) => return Poll::Ready(Some(Err(ParseError::Stream(e)))),
        };
        let parsed = ready!(this.inner.as_mut().try_poll_next(cx));
        let end = match ready!(this.inner.as_mut().borrow_mut_pin().poll_position(cx)) {
            Ok(s) => s,
            Err(e) => return Poll::Ready(Some(Err(ParseError::Stream(e)))),
        };
        match parsed {
            Some(Ok(i)) => Poll::Ready(Some(Ok(i))),
            Some(Err(err)) => match err {
                ParseError::Parser(e) => {
                    Poll::Ready(Some(Err(ParseError::Parser(PositionedError {
                        range: start..end,
                        error: e,
                    }))))
                }
                ParseError::Stream(e) => Poll::Ready(Some(Err(ParseError::Stream(e)))),
            },
            None => Poll::Ready(None),
        }
    }
}
