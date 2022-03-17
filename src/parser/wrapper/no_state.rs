use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`no_state`].
///
/// [`no_state`]: crate::parser::ParserExt::no_state
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
    ) -> PolledResult<Self::Output, I> {
        match self.inner.poll_parse(input, cx, &mut self.state) {
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
    ) -> PolledResult<Option<Self::Item>, I> {
        self.inner
            .poll_parse_next(input, cx, &mut self.state)
            .map_ok(|status| {
                if !matches!(status, Status::Success(Some(_), _)) {
                    self.state = Default::default();
                }
                status
            })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
