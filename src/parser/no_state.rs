use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use super::streamed::StreamedParser;
use crate::error::{ParseResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`no_state`].
///
/// [`no_state`]: super::ParserExt::no_state
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoState<P, C> {
    inner: P,
    state: C,
}

impl<P, C> NoState<P, C> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P) -> Self
    where
        C: Default,
    {
        Self {
            inner,
            state: Default::default(),
        }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, I> Parser<I> for NoState<P, P::State>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = ();

    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        _state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        match self.inner.poll_parse(input, cx, &mut self.state, tracker) {
            Poll::Ready(res) => {
                self.state = Default::default();
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<P, I> StreamedParser<I> for NoState<P, P::State>
where
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = ();

    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        _state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        Poll::Ready(
            match ready!(self
                .inner
                .poll_parse_next(input, cx, &mut self.state, tracker))
            {
                Ok(Some(val)) => Ok(Some(val)),
                res => {
                    self.state = Default::default();
                    res
                }
            },
        )
    }
}
