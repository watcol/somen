use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, Stream};

use super::StreamedParser;
use crate::error::ParseError;
use crate::stream::Positioned;

#[derive(Debug)]
pub struct ParserStream<'a, 'b, P: ?Sized, I: ?Sized> {
    parser: &'a mut P,
    input: &'b mut I,
}

impl<P: ?Sized, I: Unpin + ?Sized> Unpin for ParserStream<'_, '_, P, I> {}

impl<'a, 'b, P: ?Sized, I: ?Sized> ParserStream<'a, 'b, P, I> {
    pub fn new(parser: &'a mut P, input: &'b mut I) -> Self {
        Self { parser, input }
    }
}

impl<P: StreamedParser<I> + ?Sized, I: Positioned + Unpin + ?Sized> Stream
    for ParserStream<'_, '_, P, I>
{
    #[allow(clippy::type_complexity)]
    type Item = Result<P::Item, ParseError<P::Error, I::Error, I::Locator>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self {
            ref mut parser,
            ref mut input,
        } = &mut *self;
        match ready!(parser.poll_parse_next(Pin::new(input), cx)?) {
            Some(i) => Poll::Ready(Some(Ok(i))),
            None => Poll::Ready(None),
        }
    }
}
