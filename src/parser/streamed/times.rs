use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{PolledResult, Tracker};
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
    committed: bool,
    prev_committed: bool,
    stream_committed: bool,
}

impl<C: Default> Default for FlatTimesState<C> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: C::default(),
            count: 0,
            committed: false,
            prev_committed: false,
            stream_committed: false,
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
    ) -> PolledResult<Option<Self::Item>, I> {
        loop {
            if state.count >= self.count {
                break Poll::Ready(Ok((None, state.prev_committed)));
            }

            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner, tracker))
            {
                Ok((Some(val), committed)) => {
                    state.stream_committed |= committed;
                    break Poll::Ready(Ok((
                        Some(val),
                        mem::take(&mut state.prev_committed) || committed,
                    )));
                }
                Ok((None, committed)) => {
                    state.committed = mem::take(&mut state.stream_committed) | committed;
                    state.prev_committed |= committed;
                    state.count += 1;
                    state.inner = Default::default();
                }
                Err(err) => break Poll::Ready(Err(err.fatal_if(state.committed))),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner = self.inner.size_hint();
        (inner.0 * self.count, inner.1.map(|x| x * self.count))
    }
}
