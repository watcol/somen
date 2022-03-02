use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, Stream};

use super::StreamedParser;
use crate::error::{ParseError, ParseResult, Status};
use crate::stream::Positioned;

#[derive(Debug)]
pub struct ParserStream<'a, 'b, P: ?Sized, I: ?Sized, C> {
    parser: &'a mut P,
    input: &'b mut I,
    state: C,
}

impl<P: ?Sized, I: Unpin + ?Sized, C> Unpin for ParserStream<'_, '_, P, I, C> {}

impl<'a, 'b, P: StreamedParser<I> + ?Sized, I: Positioned + Unpin + ?Sized>
    ParserStream<'a, 'b, P, I, P::State>
{
    pub fn new(parser: &'a mut P, input: &'b mut I) -> Self {
        Self {
            parser,
            input,
            state: Default::default(),
        }
    }
}

impl<P: StreamedParser<I> + ?Sized, I: Positioned + Unpin + ?Sized> Stream
    for ParserStream<'_, '_, P, I, P::State>
{
    type Item = ParseResult<P::Item, I>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self {
            parser,
            input,
            ref mut state,
        } = &mut *self;
        Poll::Ready(
            match ready!(parser.poll_parse_next(Pin::new(input), cx, state)) {
                Ok((Status::Success(Some(val), _), _)) => Some(Ok(val)),
                Ok((Status::Success(None, _), _)) => None,
                Ok((Status::Fail(err, _), _)) => Some(Err(ParseError::Parser(err))),
                Err(err) => Some(Err(ParseError::Stream(err))),
            },
        )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.parser.size_hint()
    }
}
