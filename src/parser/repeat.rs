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
pub struct Repeat<P, R> {
    inner: P,
    range: R,
}

impl<P, R> Repeat<P, R> {
    /// Creating a new instance.
    #[inline]
    pub fn new<T>(inner: P, range: T) -> Self
    where
        T: RangeArgument<Target = R>,
    {
        Self {
            inner,
            range: range.into_range_bounds(),
        }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

#[derive(Debug)]
pub struct RepeatState<C, M> {
    inner: C,
    queued_marker: Option<M>,
    count: usize,
}

impl<C: Default, M> Default for RepeatState<C, M> {
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
}

/// Arguments for method [`repeat`] which is convertable to an object implements
/// [`RangeBounds`]`<usize>`.
///
/// [`repeat`]: super::ParserExt::repeat
pub trait RangeArgument {
    /// The type of converted [`RangeBounds`] object.
    type Target: RangeBounds<usize>;

    /// Convert to a [`RangeBounds`] object.
    fn into_range_bounds(self) -> Self::Target;
}

impl RangeArgument for usize {
    type Target = core::ops::RangeInclusive<usize>;
    fn into_range_bounds(self) -> Self::Target {
        self..=self
    }
}

macro_rules! impl_argument {
    ($t:ty) => {
        impl RangeArgument for $t {
            type Target = Self;

            fn into_range_bounds(self) -> Self::Target {
                self
            }
        }
    };
}

impl_argument! { core::ops::Range<usize> }
impl_argument! { core::ops::RangeInclusive<usize> }
impl_argument! { core::ops::RangeFrom<usize> }
impl_argument! { core::ops::RangeTo<usize> }
impl_argument! { core::ops::RangeToInclusive<usize> }
impl_argument! { core::ops::RangeFull }
