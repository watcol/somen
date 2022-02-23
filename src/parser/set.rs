use core::marker::PhantomData;
use core::ops::{
    Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use super::Parser;
use crate::error::{Expect, Expects, ParseError, ParseResult, Tracker};
use crate::stream::Positioned;

/// A parser for function [`one_of`].
///
/// [`one_of`]: crate::parser::one_of
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OneOf<I: ?Sized, S> {
    set: S,
    _phantom: PhantomData<I>,
}

impl<I: ?Sized, S> OneOf<I, S> {
    /// Creating a new instance.
    #[inline]
    pub fn new(set: S) -> Self {
        Self {
            set,
            _phantom: PhantomData,
        }
    }
}

impl<I, S> Parser<I> for OneOf<I, S>
where
    I: Positioned + ?Sized,
    S: Set<I::Ok>,
{
    type Output = I::Ok;
    type State = ();

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        _state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        let start = input.position();
        Poll::Ready(match ready!(input.as_mut().try_poll_next(cx)?) {
            Some(i) if self.set.contains(&i) => {
                tracker.clear();
                Ok(i)
            }
            _ => Err(ParseError::Parser {
                expects: self.set.to_expects(),
                position: start..input.position(),
                fatal: false,
            }),
        })
    }
}

/// A parser for function [`none_of`].
///
/// [`none_of`]: crate::parser::none_of
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoneOf<I: ?Sized, S> {
    set: S,
    _phantom: PhantomData<I>,
}

impl<I: ?Sized, S> NoneOf<I, S> {
    /// Creating a new instance.
    #[inline]
    pub fn new(set: S) -> Self {
        Self {
            set,
            _phantom: PhantomData,
        }
    }
}

impl<I, S> Parser<I> for NoneOf<I, S>
where
    I: Positioned + ?Sized,
    S: Set<I::Ok>,
{
    type Output = I::Ok;
    type State = ();

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        _state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        let start = input.position();
        Poll::Ready(match ready!(input.as_mut().try_poll_next(cx)?) {
            Some(i) if !self.set.contains(&i) => {
                tracker.clear();
                Ok(i)
            }
            _ => Err(ParseError::Parser {
                expects: self.set.to_expects(),
                position: start..input.position(),
                fatal: false,
            }),
        })
    }
}

/// A set for function [`one_of`], [`none_of`].
///
/// [`one_of`]: crate::parser::one_of
/// [`none_of`]: crate::parser::none_of
pub trait Set<T> {
    /// Whether the set contains `elem`, or not.
    fn contains(&self, token: &T) -> bool;

    /// Express the set by [`Expects`].
    #[inline]
    fn to_expects(&self) -> Expects<T> {
        Expects::new(Expect::Static("<set>"))
    }
}

impl<'a, T, S: Set<T> + ?Sized> Set<T> for &'a S {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        (**self).contains(token)
    }

    #[inline]
    fn to_expects(&self) -> Expects<T> {
        (**self).to_expects()
    }
}

impl<T: PartialEq + Clone> Set<T> for [T] {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        <[T]>::contains(self, token)
    }

    #[inline]
    fn to_expects(&self) -> Expects<T> {
        Expects::from_iter(self.iter().cloned().map(Expect::Token))
    }
}

impl<T: PartialEq + Clone, const N: usize> Set<T> for [T; N] {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        <[T]>::contains(self, token)
    }

    #[inline]
    fn to_expects(&self) -> Expects<T> {
        Expects::from_iter(self.iter().cloned().map(Expect::Token))
    }
}

impl Set<char> for str {
    #[inline]
    fn contains(&self, token: &char) -> bool {
        str::contains(self, *token)
    }

    #[inline]
    fn to_expects(&self) -> Expects<char> {
        Expects::from_iter(self.chars().map(Expect::Token))
    }
}

impl<T> Set<T> for RangeFull {
    #[inline]
    fn contains(&self, _: &T) -> bool {
        true
    }
}

impl<T: PartialOrd> Set<T> for (Bound<T>, Bound<T>) {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        RangeBounds::contains(self, token)
    }
}

impl<'a, T: PartialOrd> Set<T> for (Bound<&'a T>, Bound<&'a T>) {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        RangeBounds::contains(self, token)
    }
}

macro_rules! set_impl_range {
    ($t:tt) => {
        impl<T: PartialOrd> Set<T> for $t<T> {
            #[inline]
            fn contains(&self, token: &T) -> bool {
                RangeBounds::contains(self, token)
            }
        }

        impl<T: PartialOrd> Set<T> for $t<&T> {
            #[inline]
            fn contains(&self, token: &T) -> bool {
                RangeBounds::contains(self, token)
            }
        }
    };
}

set_impl_range! { Range }
set_impl_range! { RangeTo }
set_impl_range! { RangeInclusive }
set_impl_range! { RangeFrom }
set_impl_range! { RangeToInclusive }

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<T: PartialEq + Clone> Set<T> for alloc::vec::Vec<T> {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        <[T]>::contains(self, token)
    }

    #[inline]
    fn to_expects(&self) -> Expects<T> {
        Expects::from_iter(self.iter().cloned().map(Expect::Token))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<T: PartialEq + Clone> Set<T> for alloc::collections::VecDeque<T> {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        alloc::collections::VecDeque::contains(self, token)
    }

    #[inline]
    fn to_expects(&self) -> Expects<T> {
        Expects::from_iter(self.iter().cloned().map(Expect::Token))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<T: Ord + Clone> Set<T> for alloc::collections::BTreeSet<T> {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        alloc::collections::BTreeSet::contains(self, token)
    }

    #[inline]
    fn to_expects(&self) -> Expects<T> {
        Expects::from_iter(self.iter().cloned().map(Expect::Token))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<T: PartialEq + Clone> Set<T> for alloc::collections::LinkedList<T> {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        alloc::collections::LinkedList::contains(self, token)
    }

    #[inline]
    fn to_expects(&self) -> Expects<T> {
        Expects::from_iter(self.iter().cloned().map(Expect::Token))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl Set<char> for alloc::string::String {
    #[inline]
    fn contains(&self, token: &char) -> bool {
        str::contains(self, *token)
    }

    #[inline]
    fn to_expects(&self) -> Expects<char> {
        Expects::from_iter(self.chars().map(Expect::Token))
    }
}

#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
impl<T: Eq + core::hash::Hash + Clone> Set<T> for std::collections::HashSet<T> {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        std::collections::HashSet::contains(self, token)
    }

    #[inline]
    fn to_expects(&self) -> Expects<T> {
        Expects::from_iter(self.iter().cloned().map(Expect::Token))
    }
}
