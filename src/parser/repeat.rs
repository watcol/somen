use core::mem;
use core::ops::{Bound, RangeBounds};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, ParseResult, Tracker};
use crate::parser::Parser;
use crate::prelude::StreamedParser;
use crate::stream::Input;

/// A streamed parser generated from method [`repeat`].
///
/// [`repeat`]: super::ParserExt::repeat
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repeat<P, R> {
    inner: P,
    range: R,
}

impl<P, R> Repeat<P, R> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, range: R) -> Self {
        Self { inner, range }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeatState<C, M> {
    inner: C,
    queued_marker: Option<M>,
    count: usize,
}

impl<C: Default, M> Default for RepeatState<C, M> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: C::default(),
            queued_marker: None,
            count: 0,
        }
    }
}

impl<P, R, I> StreamedParser<I> for Repeat<P, R>
where
    P: Parser<I>,
    R: RangeBounds<usize>,
    I: Input + ?Sized,
{
    type Item = P::Output;
    type State = RepeatState<P::State, I::Marker>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        // Return `None` if the number of items already reached `end_bound`.
        if match self.range.end_bound() {
            Bound::Included(i) => state.count + 1 > *i,
            Bound::Excluded(i) => state.count + 1 >= *i,
            Bound::Unbounded => false,
        } {
            return Poll::Ready(Ok(None));
        }

        // Reserve the marker.
        if state.queued_marker.is_none() {
            state.queued_marker = Some(input.as_mut().mark()?);
        }

        Poll::Ready(
            match ready!(self
                .inner
                .poll_parse(input.as_mut(), cx, &mut state.inner, tracker))
            {
                Ok(output) => {
                    input
                        .as_mut()
                        .drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    state.inner = Default::default();
                    state.count += 1;
                    Ok(Some(output))
                }
                // Return `None` if `count` already satisfies the minimal bound.
                Err(ParseError::Parser {
                    fatal: false,
                    expects,
                    ..
                }) if self.range.contains(&state.count) => {
                    input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    tracker.add(expects);
                    Ok(None)
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    // If the parser has succeeded parsing at least once, rewinding the parser is
                    // not appropriate.
                    Err(if state.count > 0 {
                        err.fatal(true)
                    } else {
                        err
                    })
                }
            },
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let start = match self.range.start_bound() {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => *i + 1,
            Bound::Unbounded => 0,
        };

        let end = match self.range.end_bound() {
            Bound::Included(i) => Some(*i),
            Bound::Excluded(i) => Some(*i - 1),
            Bound::Unbounded => None,
        };

        (start, end)
    }
}
