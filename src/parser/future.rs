use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use super::Parser;
use crate::error::{ParseError, ParseResult, Tracker};
use crate::stream::Positioned;

#[derive(Debug)]
pub struct ParseFuture<'a, 'b, P: ?Sized, I: ?Sized, C, T> {
    parser: &'a mut P,
    input: &'b mut I,
    state: C,
    tracker: Tracker<T>,
}

impl<P: ?Sized, I: Unpin + ?Sized, C, T> Unpin for ParseFuture<'_, '_, P, I, C, T> {}

impl<'a, 'b, P: Parser<I> + ?Sized, I: Positioned + Unpin + ?Sized>
    ParseFuture<'a, 'b, P, I, P::State, I::Ok>
{
    pub fn new(parser: &'a mut P, input: &'b mut I) -> Self {
        Self {
            parser,
            input,
            state: Default::default(),
            tracker: Tracker::default(),
        }
    }
}

impl<P: Parser<I> + ?Sized, I: Positioned + Unpin + ?Sized> Future
    for ParseFuture<'_, '_, P, I, P::State, I::Ok>
{
    type Output = ParseResult<P::Output, I>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self {
            parser,
            input,
            ref mut state,
            ref mut tracker,
        } = &mut *self;
        Poll::Ready(
            match ready!(parser.poll_parse(Pin::new(input), cx, state, tracker)) {
                Ok((val, _)) => Ok(val),
                Err(ParseError::Parser {
                    expects,
                    position,
                    fatal,
                }) => Err(ParseError::Parser {
                    expects: expects.merge(tracker.clear()),
                    position,
                    fatal,
                }),
                Err(err) => Err(err),
            },
        )
    }
}
