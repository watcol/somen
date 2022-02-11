use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{ParseResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`no_state`].
///
/// [`no_state`]: super::ParserExt::no_state
#[derive(Debug)]
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

#[derive(Debug)]
pub struct LazyState<P, C> {
    parser: Option<P>,
    inner: C,
}

impl<P, C: Default> Default for LazyState<P, C> {
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
