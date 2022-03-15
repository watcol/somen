use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use super::Parser;
use crate::error::{ParseError, ParseResult, Status};
use crate::stream::Positioned;

#[derive(Debug)]
pub struct ParseFuture<'a, 'b, P: ?Sized, I: ?Sized, C> {
    parser: &'a mut P,
    input: &'b mut I,
    state: C,
}

impl<P: ?Sized, I: Unpin + ?Sized, C> Unpin for ParseFuture<'_, '_, P, I, C> {}

impl<'a, 'b, P: Parser<I> + ?Sized, I: Positioned + Unpin + ?Sized>
    ParseFuture<'a, 'b, P, I, P::State>
{
    pub fn new(parser: &'a mut P, input: &'b mut I) -> Self {
        Self {
            parser,
            input,
            state: Default::default(),
        }
    }
}

impl<P: Parser<I> + ?Sized, I: Positioned + Unpin + ?Sized> Future
    for ParseFuture<'_, '_, P, I, P::State>
{
    type Output = ParseResult<P::Output, I>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self {
            parser,
            input,
            ref mut state,
        } = &mut *self;
        Poll::Ready(
            match ready!(parser.poll_parse(Pin::new(input), cx, state)) {
                Ok((Status::Success(val, _), _)) => Ok(val),
                Ok((Status::Failure(err, _), _)) => Err(ParseError::Parser(err)),
                Err(err) => Err(ParseError::Stream(err)),
            },
        )
    }
}
