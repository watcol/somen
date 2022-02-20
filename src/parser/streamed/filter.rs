use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseResult, Tracker};
use crate::parser::streamed::StreamedParser;
use crate::stream::Positioned;

/// A parser for method [`filter`].
///
/// [`filter`]: super::StreamedParserExt::filter
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter<P, F> {
    inner: P,
    f: F,
}

impl<P, F> Filter<P, F> {
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

impl<P, F, I> StreamedParser<I> for Filter<P, F>
where
    P: StreamedParser<I>,
    F: FnMut(&P::Item) -> bool,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = P::State;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        loop {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, state, tracker)?)
            {
                Some(val) if (self.f)(&val) => break Poll::Ready(Ok(Some(val))),
                Some(_) => continue,
                None => break Poll::Ready(Ok(None)),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.inner.size_hint().1)
    }
}
