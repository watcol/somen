use core::fmt;

use super::{Expect, ExpectKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expects<T>(Expect<T>);

impl<T> From<Expect<T>> for Expects<T> {
    #[inline]
    fn from(inner: Expect<T>) -> Self {
        Self(inner)
    }
}

impl<T: fmt::Display> fmt::Display for Expects<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> IntoIterator for Expects<T> {
    type Item = Expect<T>;
    type IntoIter = core::iter::Once<Expect<T>>;

    fn into_iter(self) -> Self::IntoIter {
        core::iter::once(self.0)
    }
}

impl<T> FromIterator<Expect<T>> for Expects<T> {
    fn from_iter<I: IntoIterator<Item = Expect<T>>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        match iter.next() {
            Some(ex) if iter.next().is_none() => Self(ex),
            _ => Self(Expect::Positive(ExpectKind::Other)),
        }
    }
}

impl<T> Expects<T> {
    #[inline]
    #[allow(unused_variables)]
    pub fn merge(self, other: Expects<T>) -> Self {
        Self(Expect::Positive(ExpectKind::Other))
    }

    /// Converting each elements.
    #[inline]
    pub fn map<F: FnMut(Expect<T>) -> Expect<U>, U>(self, mut f: F) -> Expects<U> {
        Expects(f(self.0))
    }

    #[inline]
    pub fn sort(&mut self)
    where
        T: Ord,
    {
    }
}
