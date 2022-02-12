use core::ops::{
    Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

use crate::error::{Expect, Expects};

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

impl<'a, T, S: Set<T>> Set<T> for &'a S {
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
