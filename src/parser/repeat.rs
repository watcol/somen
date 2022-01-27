use core::fmt;
use core::mem;
use core::ops::{Bound, RangeBounds};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, StreamedResult};
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

/// An error type for method [`repeat`].
///
/// This error will returned when the number of items is not enough as the lower bound.
///
/// [`repeat`]: super::ParserExt::repeat
#[derive(Debug)]
pub struct RepeatError<E> {
    /// An error from the internal parser.
    pub inner: E,
    /// The number of succeeded items.
    pub suc_count: usize,
    /// The minimum bound for the number of items.
    pub min_bound: usize,
}

impl<E: fmt::Display> fmt::Display for RepeatError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(feature = "std")]
impl<E: std::error::Error + 'static> std::error::Error for RepeatError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
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
    type Error = RepeatError<P::Error>;
    type State = RepeatState<P::State, I::Marker>;

    fn poll_parse_next(
        &self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<StreamedResult<Self, I>> {
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
            state.queued_marker = Some(input.as_mut().mark().map_err(ParseError::Stream)?);
        }

        Poll::Ready(
            match ready!(self.inner.poll_parse(input.as_mut(), cx, &mut state.inner)) {
                Ok(output) => {
                    state.count += 1;
                    Ok(Some(output))
                }
                // Return `None` if `count` already satisfies the minimal bound.
                Err(ParseError::Parser(_, _)) if self.range.contains(&state.count) => {
                    input
                        .rewind(mem::take(&mut state.queued_marker).unwrap())
                        .map_err(ParseError::Stream)?;
                    Ok(None)
                }
                // else, the parser returns an error.
                Err(ParseError::Parser(e, p)) => Err(ParseError::Parser(
                    RepeatError {
                        inner: e,
                        suc_count: state.count,
                        min_bound: match self.range.start_bound() {
                            Bound::Included(i) => *i,
                            Bound::Excluded(i) => *i - 1,
                            Bound::Unbounded => 0,
                        },
                    },
                    p,
                )),
                Err(ParseError::Stream(e)) => return Poll::Ready(Err(ParseError::Stream(e))),
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
