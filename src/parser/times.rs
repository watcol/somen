use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use super::streamed::StreamedParser;
use crate::error::{ParseResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A streamed parser generated from method [`times`] and [`once`].
///
/// [`once`]: super::ParserExt::once
/// [`times`]: super::ParserExt::times
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Times<P> {
    inner: P,
    count: usize,
}

impl<P> Times<P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, count: usize) -> Self {
        Self { inner, count }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimesState<C> {
    inner: C,
    count: usize,
}

impl<C: Default> Default for TimesState<C> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: C::default(),
            count: 0,
        }
    }
}

impl<P, I> StreamedParser<I> for Times<P>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Output;
    type State = TimesState<P::State>;

    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        if state.count >= self.count {
            return Poll::Ready(Ok(None));
        }

        Poll::Ready(
            match ready!(self.inner.poll_parse(input, cx, &mut state.inner, tracker)) {
                Ok(i) => {
                    state.count += 1;
                    state.inner = Default::default();
                    Ok(Some(i))
                }
                Err(err) if state.count == 0 => Err(err),
                Err(err) => Err(err.fatal(true)),
            },
        )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}
