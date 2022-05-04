#[cfg(feature = "alloc")]
use alloc::{format, string::ToString};
#[cfg(feature = "alloc")]
use core::fmt::Display;
use core::marker::PhantomData;
use core::ops::{
    Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::Parser;
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
    /// Creates a new instance.
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
    ) -> PolledResult<Self::Output, I> {
        let start = input.position();
        Poll::Ready(Ok(match ready!(input.as_mut().try_poll_next(cx)?) {
            Some(i) if self.set.contains(&i) => Status::Success(i, None),
            _ => Status::Failure(
                Error {
                    expects: self.set.to_expects(),
                    position: start..input.position(),
                },
                false,
            ),
        }))
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
    /// Creates a new instance.
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
    ) -> PolledResult<Self::Output, I> {
        let start = input.position();
        Poll::Ready(Ok(match ready!(input.as_mut().try_poll_next(cx)?) {
            Some(i) if !self.set.contains(&i) => Status::Success(i, None),
            _ => Status::Failure(
                Error {
                    #[cfg(feature = "alloc")]
                    expects: Expects::from_iter(
                        self.set
                            .to_expects()
                            .into_iter()
                            .map(|exp| alloc::borrow::Cow::from(format!("not {}", exp))),
                    ),
                    #[cfg(not(feature = "alloc"))]
                    expects: Expects::from("<none of set>"),
                    position: start..input.position(),
                },
                false,
            ),
        }))
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
    fn to_expects(&self) -> Expects {
        Expects::from("<set>")
    }
}

impl<'a, T, S: Set<T> + ?Sized> Set<T> for &'a S {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        (**self).contains(token)
    }

    #[inline]
    fn to_expects(&self) -> Expects {
        (**self).to_expects()
    }
}

impl<
        T,
        #[cfg(not(feature = "alloc"))] U: PartialEq<T>,
        #[cfg(feature = "alloc")] U: PartialEq<T> + Display,
    > Set<T> for [U]
{
    #[inline]
    fn contains(&self, token: &T) -> bool {
        self.iter().any(|i| i == token)
    }

    #[inline]
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    fn to_expects(&self) -> Expects {
        Expects::from_iter(self.iter().map(|t| t.to_string()))
    }
}

impl<
        T,
        #[cfg(not(feature = "alloc"))] U: PartialEq<T>,
        #[cfg(feature = "alloc")] U: PartialEq<T> + Display,
        const N: usize,
    > Set<T> for [U; N]
{
    #[inline]
    fn contains(&self, token: &T) -> bool {
        self.iter().any(|i| i == token)
    }

    #[inline]
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    fn to_expects(&self) -> Expects {
        Expects::from_iter(self.iter().map(|t| t.to_string()))
    }
}

impl<T: PartialEq<char>> Set<T> for str {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        self.chars().any(|c| *token == c)
    }

    #[inline]
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    fn to_expects(&self) -> Expects {
        Expects::from_iter(self.chars().map(|t| t.to_string()))
    }
}

impl<T> Set<T> for RangeFull {
    #[inline]
    fn contains(&self, _: &T) -> bool {
        true
    }
}

impl<T: PartialOrd<U>, U: PartialOrd<T>> Set<T> for (Bound<U>, Bound<U>) {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        RangeBounds::contains(self, token)
    }
}

macro_rules! set_impl_range {
    ($t:tt) => {
        impl<T: PartialOrd<U>, U: PartialOrd<T>> Set<T> for $t<U> {
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
impl<T, U: PartialEq<T> + Display> Set<T> for alloc::vec::Vec<U> {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        self.iter().any(|i| i == token)
    }

    #[inline]
    fn to_expects(&self) -> Expects {
        Expects::from_iter(self.iter().map(|t| t.to_string()))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<T, U: PartialEq<T> + Display> Set<T> for alloc::collections::VecDeque<U> {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        self.iter().any(|i| i == token)
    }

    #[inline]
    fn to_expects(&self) -> Expects {
        Expects::from_iter(self.iter().map(|t| t.to_string()))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<T, U: PartialEq<T> + Ord + Display> Set<T> for alloc::collections::BTreeSet<U> {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        self.iter().any(|i| i == token)
    }

    #[inline]
    fn to_expects(&self) -> Expects {
        Expects::from_iter(self.iter().map(|t| t.to_string()))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<T, U: PartialEq<T> + Display> Set<T> for alloc::collections::LinkedList<U> {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        self.iter().any(|i| i == token)
    }

    #[inline]
    fn to_expects(&self) -> Expects {
        Expects::from_iter(self.iter().map(|t| t.to_string()))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<T: PartialEq<char>> Set<T> for alloc::string::String {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        self.chars().any(|c| *token == c)
    }

    #[inline]
    fn to_expects(&self) -> Expects {
        Expects::from_iter(self.chars().map(|t| t.to_string()))
    }
}

#[cfg(feature = "std")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "std")))]
impl<T, U: PartialEq<T> + Eq + core::hash::Hash + Display> Set<T> for std::collections::HashSet<U> {
    #[inline]
    fn contains(&self, token: &T) -> bool {
        self.iter().any(|i| i == token)
    }

    #[inline]
    fn to_expects(&self) -> Expects {
        Expects::from_iter(self.iter().map(|t| t.to_string()))
    }
}
