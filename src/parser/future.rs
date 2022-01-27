use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::Parser;
use crate::error::ParseResult;
use crate::stream::Positioned;

#[derive(Debug)]
pub struct ParseFuture<'a, 'b, P: ?Sized, I: ?Sized, C> {
    parser: &'a P,
    input: &'b mut I,
    state: C,
}

impl<P: ?Sized, I: Unpin + ?Sized, C> Unpin for ParseFuture<'_, '_, P, I, C> {}

impl<'a, 'b, P: Parser<I> + ?Sized, I: Positioned + Unpin + ?Sized>
    ParseFuture<'a, 'b, P, I, P::State>
{
    pub fn new(parser: &'a P, input: &'b mut I) -> Self {
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
    type Output = ParseResult<P, I>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self {
            parser,
            input,
            ref mut state,
        } = &mut *self;
        parser.poll_parse(Pin::new(input), cx, state)
    }
}
