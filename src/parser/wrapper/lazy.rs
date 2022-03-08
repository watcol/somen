use core::pin::Pin;
use core::task::Context;

use crate::error::PolledResult;
use crate::parser::streamed::StreamedParser;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`lazy`].
///
/// [`lazy`]: crate::parser::lazy
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lazy<F> {
    f: F,
}

impl<F> Lazy<F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

crate::parser_state! {
    pub struct LazyState<I, P: Parser> {
        #[state(option)]
        parser: P,
        inner: P::State,
    }
}

impl<F, P, I> Parser<I> for Lazy<F>
where
    F: FnMut() -> P,
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = LazyState<I, P>;

    #[inline]
    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        state
            .parser
            .get_or_insert_with(&mut self.f)
            .poll_parse(input, cx, &mut state.inner)
    }
}

crate::parser_state! {
    pub struct LazyStreamedState<I, P: StreamedParser> {
        #[state(option)]
        parser: P,
        inner: P::State,
    }
}

impl<F, P, I> StreamedParser<I> for Lazy<F>
where
    F: FnMut() -> P,
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = LazyStreamedState<I, P>;

    #[inline]
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        state
            .parser
            .get_or_insert_with(&mut self.f)
            .poll_parse_next(input, cx, &mut state.inner)
    }
}
