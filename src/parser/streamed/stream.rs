use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, Stream};

use super::StreamedParser;
use crate::error::{ParseError, ParseResult, Tracker};
use crate::stream::Positioned;

#[derive(Debug)]
pub struct ParserStream<'a, 'b, P: ?Sized, I: ?Sized, C, T> {
    parser: &'a mut P,
    input: &'b mut I,
    state: C,
    tracker: Tracker<T>,
}

impl<P: ?Sized, I: Unpin + ?Sized, C, T> Unpin for ParserStream<'_, '_, P, I, C, T> {}

impl<'a, 'b, P: StreamedParser<I> + ?Sized, I: Positioned + Unpin + ?Sized>
    ParserStream<'a, 'b, P, I, P::State, I::Ok>
{
    pub fn new(parser: &'a mut P, input: &'b mut I) -> Self {
        Self {
            parser,
            input,
            state: Default::default(),
            tracker: Tracker::new(),
        }
    }
}

impl<P: StreamedParser<I> + ?Sized, I: Positioned + Unpin + ?Sized> Stream
    for ParserStream<'_, '_, P, I, P::State, I::Ok>
{
    type Item = ParseResult<P::Item, I>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self {
            parser,
            input,
            ref mut state,
            ref mut tracker,
        } = &mut *self;
        match ready!(parser
            .poll_parse_next(Pin::new(input), cx, state, tracker)
            .map_err(|err| match err {
                ParseError::Parser {
                    expects,
                    position,
                    fatal,
                } => ParseError::Parser {
                    expects: expects.merge(tracker.clear()),
                    position,
                    fatal,
                },
                err => err,
            })?) {
            Some(i) => Poll::Ready(Some(Ok(i))),
            None => Poll::Ready(None),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.parser.size_hint()
    }
}
