use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::stream::TryStream;

use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::stream::position::Positioned;

pub struct ParseFuture<'a, 'b, P, S: ?Sized> {
    parser: &'a P,
    stream: &'b mut S,
}

impl<'a, 'b, P, S> ParseFuture<'a, 'b, P, S>
where
    P: Parser,
    S: TryStream<Ok = P::Item>
        + Positioned<Position = <P::Error as ParseError>::Position>
        + ?Sized
        + Unpin,
{
    pub fn new(parser: &'a P, stream: &'b mut S) -> Self {
        Self { parser, stream }
    }
}

impl<P, S: ?Sized + Unpin> Unpin for ParseFuture<'_, '_, P, S> {}

impl<P, S> Future for ParseFuture<'_, '_, P, S>
where
    P: Parser,
    S: TryStream<Ok = P::Item>
        + Positioned<Position = <P::Error as ParseError>::Position>
        + ?Sized
        + Unpin,
{
    type Output = ParseResult<P, S>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.parser.poll_parse(Pin::new(&mut *self.stream), cx)
    }
}
