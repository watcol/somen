use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::stream::TryStream;

use crate::error::{ParseResult, PositionedResult};
use crate::parser::Parser;
use crate::stream::position::Positioned;

pub struct ParseFuture<'a, 'b, P: ?Sized, I: ?Sized> {
    parser: &'a P,
    input: &'b mut I,
}

impl<'a, 'b, P, I> ParseFuture<'a, 'b, P, I>
where
    P: Parser<I> + ?Sized,
    I: TryStream + ?Sized + Unpin,
{
    pub fn new(parser: &'a P, input: &'b mut I) -> Self {
        Self { parser, input }
    }
}

impl<P: ?Sized, I: ?Sized + Unpin> Unpin for ParseFuture<'_, '_, P, I> {}

impl<P, I> Future for ParseFuture<'_, '_, P, I>
where
    P: Parser<I> + ?Sized,
    I: TryStream + ?Sized + Unpin,
{
    type Output = ParseResult<P, I>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.parser.poll_parse(Pin::new(&mut *self.input), cx)
    }
}

pub struct ParsePositionedFuture<'a, 'b, P: ?Sized, I: ?Sized> {
    parser: &'a P,
    input: &'b mut I,
}

impl<'a, 'b, P, I> ParsePositionedFuture<'a, 'b, P, I>
where
    P: Parser<I> + ?Sized,
    I: TryStream + Positioned + ?Sized + Unpin,
{
    pub fn new(parser: &'a P, input: &'b mut I) -> Self {
        Self { parser, input }
    }
}

impl<P: ?Sized, I: ?Sized + Unpin> Unpin for ParsePositionedFuture<'_, '_, P, I> {}

impl<P, I> Future for ParsePositionedFuture<'_, '_, P, I>
where
    P: Parser<I> + ?Sized,
    I: TryStream + Positioned + ?Sized + Unpin,
{
    type Output = PositionedResult<P, I>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.parser
            .poll_parse_positioned(Pin::new(&mut *self.input), cx)
    }
}
