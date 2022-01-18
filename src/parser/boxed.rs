use super::Parser;
use crate::error::ParseResult;
use crate::stream::BasicInput;

use core::pin::Pin;
use core::task::{Context, Poll};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[cfg(feature = "alloc")]
pub type BoxParser<'a, I, O, E> = Box<&'a dyn Parser<I, Output = O, Error = E>>;

#[cfg(feature = "alloc")]
impl<I: BasicInput + ?Sized, O, E> Parser<I> for BoxParser<'_, I, O, E> {
    type Output = O;
    type Error = E;

    #[inline]
    fn poll_parse(&self, input: Pin<&mut I>, cx: &mut Context<'_>) -> Poll<ParseResult<Self, I>> {
        (**self).poll_parse(input, cx)
    }
}
