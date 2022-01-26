use core::fmt;
use core::mem;
use core::ops::{Bound, RangeBounds};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, StreamedResult};
use crate::parser::Parser;
use crate::prelude::StreamedParser;
use crate::stream::{Input, Rewind};

/// A streamed parser generated from method [`repeat`].
///
/// [`repeat`]: super::ParserExt::repeat
pub struct Repeat<P, I: Rewind + ?Sized, R> {
    inner: P,
    range: R,
    queued_marker: Option<I::Marker>,
    count: usize,
}

impl<P, I: Rewind + ?Sized, R> Repeat<P, I, R> {
    /// Creating a new instance.
    #[inline]
    pub fn new<T>(inner: P, range: T) -> Self
    where
        T: RangeArgument<Target = R>,
    {
        Self {
            inner,
            range: range.into_range_bounds(),
            queued_marker: None,
            count: 0,
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

impl<P, I, R> StreamedParser<I> for Repeat<P, I, R>
where
    P: Parser<I>,
    R: RangeBounds<usize>,
    I: Input + ?Sized,
{
    type Item = P::Output;
    type Error = RepeatError<P::Error>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
    ) -> Poll<StreamedResult<Self, I>> {
        // Return `None` if the number of items already reached `end_bound`.
        if match self.range.end_bound() {
            Bound::Included(i) => self.count + 1 > *i,
            Bound::Excluded(i) => self.count + 1 >= *i,
            Bound::Unbounded => false,
        } {
            return Poll::Ready(Ok(None));
        }

        // Reserve the marker.
        if self.queued_marker.is_none() {
            self.queued_marker = Some(input.as_mut().mark().map_err(ParseError::Stream)?);
        }

        Poll::Ready(match ready!(self.inner.poll_parse(input.as_mut(), cx)) {
            Ok(output) => {
                self.count += 1;
                Ok(Some(output))
            }
            // Return `None` if `count` already satisfies the minimal bound.
            Err(ParseError::Parser(_, _)) if self.range.contains(&self.count) => {
                input
                    .rewind(mem::take(&mut self.queued_marker).unwrap())
                    .map_err(ParseError::Stream)?;
                Ok(None)
            }
            // else, the parser returns an error.
            Err(ParseError::Parser(e, p)) => Err(ParseError::Parser(
                RepeatError {
                    inner: e,
                    suc_count: self.count,
                    min_bound: match self.range.start_bound() {
                        Bound::Included(i) => *i,
                        Bound::Excluded(i) => *i - 1,
                        Bound::Unbounded => 0,
                    },
                },
                p,
            )),
            Err(ParseError::Stream(e)) => return Poll::Ready(Err(ParseError::Stream(e))),
        })
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
