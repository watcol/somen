use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use super::{Expect, ExpectKind};

/// A set of expected tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expects<T>(Vec<Expect<T>>);

impl<T> From<Expect<T>> for Expects<T> {
    #[inline]
    fn from(inner: Expect<T>) -> Self {
        Self(alloc::vec![inner])
    }
}

impl<T> From<String> for Expects<T> {
    #[inline]
    fn from(msg: String) -> Self {
        Expects::from(ExpectKind::Owned(msg))
    }
}

impl<T: fmt::Display> fmt::Display for Expects<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.0.len();
        if len == 0 {
            Ok(())
        } else {
            for (c, i) in self.0.iter().enumerate() {
                if c == 0 {
                    write!(f, "{}", i)?;
                } else if c == len - 1 {
                    write!(f, " or {}", i)?;
                } else {
                    write!(f, ", {}", i)?;
                }
            }
            Ok(())
        }
    }
}

impl<T> FromIterator<Expect<T>> for Expects<T> {
    fn from_iter<I: IntoIterator<Item = Expect<T>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T> IntoIterator for Expects<T> {
    type Item = Expect<T>;
    type IntoIter = alloc::vec::IntoIter<Expect<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Expects<T> {
    /// Merges two sets.
    pub fn merge(mut self, mut other: Expects<T>) -> Self {
        self.0.append(&mut other.0);
        self
    }

    /// Converts each elements.
    pub fn map<F: FnMut(Expect<T>) -> Expect<U>, U>(self, mut f: F) -> Expects<U> {
        Expects(self.into_iter().map(&mut f).collect())
    }

    /// Sorts and removes duplicates.
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.0.sort_unstable();
        self.0.dedup();
    }
}
