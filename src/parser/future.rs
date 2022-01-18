use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::stream::TryStream;

use crate::error::ParseResult;
use crate::parser::Parser;

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
