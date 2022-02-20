use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseResult, Tracker};
use crate::parser::streamed::StreamedParser;
use crate::stream::Positioned;

/// A parser for method [`flatten`].
///
/// [`flatten`]: super::StreamedParserExt::flatten
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flatten<P> {
    inner: P,
}

impl<P> Flatten<P> {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlattenState<C, I> {
    inner: C,
    iter: Option<I>,
}

impl<C: Default, I> Default for FlattenState<C, I> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
            iter: None,
        }
    }
}

impl<P, I> StreamedParser<I> for Flatten<P>
where
    P: StreamedParser<I>,
    P::Item: IntoIterator,
    I: Positioned + ?Sized,
{
    type Item = <P::Item as IntoIterator>::Item;
    type State = FlattenState<P::State, <P::Item as IntoIterator>::IntoIter>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        loop {
            if let Some(iter) = &mut state.iter {
                if let Some(val) = iter.next() {
                    break Poll::Ready(Ok(Some(val)));
                }
            }

            match ready!(self.inner.poll_parse_next(
                input.as_mut(),
                cx,
                &mut state.inner,
                tracker
            )?) {
                Some(val) => state.iter = Some(val.into_iter()),
                None => break Poll::Ready(Ok(None)),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.inner.size_hint().1)
    }
}
