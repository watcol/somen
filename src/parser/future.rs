use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::Parser;
use crate::error::ParseResult;
use crate::stream::Positioned;

#[derive(Debug)]
pub struct ParseFuture<'a, 'b, P: ?Sized, I: ?Sized> {
    parser: &'a mut P,
    input: &'b mut I,
}

impl<P: ?Sized, I: Unpin + ?Sized> Unpin for ParseFuture<'_, '_, P, I> {}

impl<'a, 'b, P: ?Sized, I: ?Sized> ParseFuture<'a, 'b, P, I> {
    pub fn new(parser: &'a mut P, input: &'b mut I) -> Self {
        Self { parser, input }
    }
}

impl<P: Parser<I> + ?Sized, I: Positioned + Unpin + ?Sized> Future for ParseFuture<'_, '_, P, I> {
    type Output = ParseResult<P, I>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self {
            ref mut parser,
            ref mut input,
        } = &mut *self;
        parser.poll_parse(Pin::new(input), cx)
    }
}
