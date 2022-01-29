use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::ParseResult;
use crate::parser::Parser;
use crate::stream::Input;

/// A parser for method [`map`].
///
/// [`map`]: super::Parser::map
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
    F: Fn(P::Output) -> O,
    I: Input + ?Sized,
{
    type Output = O;
    type Error = P::Error;
    type State = P::State;

    fn poll_parse(
        &self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        self.inner.poll_parse(input, cx, state).map_ok(&self.f)
    }
}
