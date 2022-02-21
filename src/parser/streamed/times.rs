use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseResult, Tracker};
use crate::parser::streamed::StreamedParser;
use crate::stream::Positioned;

/// A parser for method [`flat_times`].
///
/// [`flat_times`]: super::StreamedParserExt::flat_times
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatTimes<P> {
    inner: P,
    count: usize,
}

impl<P> FlatTimes<P> {
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
pub struct FlatTimesState<C> {
    inner: C,
    count: usize,
    consumed: bool,
}

impl<C: Default> Default for FlatTimesState<C> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: C::default(),
            count: 0,
            consumed: false,
        }
    }
}

impl<P, I> StreamedParser<I> for FlatTimes<P>
where
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = FlatTimesState<P::State>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        loop {
            if state.count >= self.count {
                break Poll::Ready(Ok(None));
            }

            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner, tracker))
            {
                Ok(Some(val)) => {
                    state.consumed = true;
                    break Poll::Ready(Ok(Some(val)));
                }
                Ok(None) => {
                    state.count += 1;
                    state.inner = Default::default();
                }
                Err(err) if !state.consumed => break Poll::Ready(Err(err)),
                Err(err) => break Poll::Ready(Err(err.fatal(true))),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner = self.inner.size_hint();
        (inner.0 * self.count, inner.1.map(|x| x * self.count))
    }
}
