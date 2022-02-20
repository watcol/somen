use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{ParseResult, Tracker};
use crate::stream::Positioned;

use super::StreamedParser;

/// A parser for method [`enumerate`].
///
/// [`enumerate`]: super::StreamedParserExt::enumerate
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Enumerate<P> {
    inner: P,
}

impl<P> Enumerate<P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P) -> Self {
        Self { inner }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Debug, Default)]
pub struct EnumerateState<C> {
    inner: C,
    count: usize,
}

impl<P, I> StreamedParser<I> for Enumerate<P>
where
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Item = (usize, P::Item);
    type State = EnumerateState<P::State>;

    #[inline]
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        self.inner
            .poll_parse_next(input, cx, &mut state.inner, tracker)
            .map_ok(|val| {
                val.map(|i| {
                    let count = state.count;
                    state.count += 1;
                    (count, i)
                })
            })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
