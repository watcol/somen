use core::convert::Infallible;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::StreamedResult;
use crate::parser::Parser;
use crate::prelude::StreamedParser;
use crate::stream::{Input, Rewind};

use super::Opt;

/// A streamed parser generated from method [`repeat`].
///
/// [`repeat`]: super::ParserExt::repeat
pub struct Repeat<P, I: Rewind + ?Sized> {
    inner: Opt<P, I>,
}

impl<P, I: Rewind + ?Sized> Repeat<P, I> {
    /// Creating a new instance.
    #[inline]
    pub fn new(parser: P) -> Self {
        Self {
            inner: Opt::new(parser),
        }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner.into_inner()
    }
}

impl<P, I> StreamedParser<I> for Repeat<P, I>
where
    P: Parser<I>,
    I: Input + ?Sized,
{
    type Item = P::Output;
    type Error = Infallible;

    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
    ) -> Poll<StreamedResult<Self, I>> {
        self.inner.poll_parse(input, cx)
    }
}
