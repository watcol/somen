use core::convert::Infallible;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::stream::Input;

/// A parser for method [`opt`].
///
/// [`opt`]: super::ParserExt::opt
#[derive(Debug)]
pub struct Opt<P, M> {
    inner: P,
    queued_marker: Option<M>,
}

impl<P, M> Opt<P, M> {
    /// Creating a new instance.
    #[inline]
    pub fn new(parser: P) -> Self {
        Self {
            inner: parser,
            queued_marker: None,
        }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, I> Parser<I> for Opt<P, I::Marker>
where
    P: Parser<I>,
    I: Input + ?Sized,
{
    type Output = Option<P::Output>;
    type Error = Infallible;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
    ) -> Poll<ParseResult<Self, I>> {
        if self.queued_marker.is_none() {
            self.queued_marker = Some(input.as_mut().mark().map_err(ParseError::Stream)?);
        }

        Poll::Ready(Ok(
            match ready!(self.inner.poll_parse(input.as_mut(), cx)) {
                Ok(i) => {
                    input
                        .drop_marker(mem::take(&mut self.queued_marker).unwrap())
                        .map_err(ParseError::Stream)?;
                    Some(i)
                }
                Err(_) => {
                    input
                        .rewind(mem::take(&mut self.queued_marker).unwrap())
                        .map_err(ParseError::Stream)?;
                    None
                }
            },
        ))
    }
}
