use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::ParseResult;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`map`].
///
/// [`map`]: super::ParserExt::map
#[derive(Debug)]
pub struct Map<P, F> {
    inner: P,
    f: F,
}

impl<P, F> Map<P, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, F, I, O> Parser<I> for Map<P, F>
where
    P: Parser<I>,
    F: FnMut(P::Output) -> O,
    I: Positioned + ?Sized,
{
    type Output = O;
    type State = P::State;

    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self::Output, I>> {
        self.inner.poll_parse(input, cx, state).map_ok(&mut self.f)
    }
}
