use core::pin::Pin;
use core::task::{Context, Poll};

use super::streamed::StreamedParser;
use crate::error::{ParseResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`no_state`].
///
/// [`no_state`]: super::ParserExt::no_state
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LazyState<P, C> {
    parser: Option<P>,
    inner: C,
}

impl<P, C: Default> Default for LazyState<P, C> {
    #[inline]
    fn default() -> Self {
        Self {
            parser: None,
            inner: Default::default(),
        }
    }
}

impl<F, P, I> Parser<I> for Lazy<F>
where
    F: FnMut() -> P,
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = LazyState<P, P::State>;

    #[inline]
    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        state.parser.get_or_insert_with(&mut self.f).poll_parse(
            input,
            cx,
            &mut state.inner,
            tracker,
        )
    }
}

impl<F, P, I> StreamedParser<I> for Lazy<F>
where
    F: FnMut() -> P,
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = LazyState<P, P::State>;

    #[inline]
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        state
            .parser
            .get_or_insert_with(&mut self.f)
            .poll_parse_next(input, cx, &mut state.inner, tracker)
    }
}
