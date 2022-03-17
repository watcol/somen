use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, Stream};

use super::StreamedParser;
use crate::error::{ParseError, ParseResult, Status};
use crate::stream::{Input, Positioned, Rewind};

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
                Ok(Status::Success(Some(val), _)) => Some(Ok(val)),
                Ok(Status::Success(None, _)) => None,
                Ok(Status::Failure(err, _)) => Some(Err(ParseError::Parser(err))),
                Err(err) => Some(Err(ParseError::Stream(err))),
            },
        )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.parser.size_hint()
    }
}

impl<P: StreamedParser<I> + ?Sized, I: Positioned + Unpin + ?Sized> Positioned
    for ParserStream<'_, '_, P, I, P::State>
{
    type Locator = I::Locator;

    #[inline]
    fn position(&self) -> Self::Locator {
        self.input.position()
    }
}

impl<P: StreamedParser<I> + ?Sized, I: Input + Unpin + ?Sized> Rewind
    for ParserStream<'_, '_, P, I, P::State>
{
    type Marker = I::Marker;

    #[inline]
    fn mark(mut self: Pin<&mut Self>) -> Result<Self::Marker, Self::Error> {
        Pin::new(&mut *self.input)
            .mark()
            .map_err(ParseError::Stream)
    }

    fn rewind(mut self: Pin<&mut Self>, marker: Self::Marker) -> Result<(), Self::Error> {
        Pin::new(&mut *self.input)
            .rewind(marker)
            .map_err(ParseError::Stream)
    }
}
