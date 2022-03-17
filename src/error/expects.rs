mod expect;
#[cfg(not(feature = "alloc"))]
mod no_std;
#[cfg(feature = "alloc")]
mod std;

#[cfg(feature = "alloc")]
pub use self::std::Expects;
pub use expect::{Expect, ExpectKind};
#[cfg(not(feature = "alloc"))]
pub use no_std::Expects;

impl<T> From<ExpectKind<T>> for Expects<T> {
    #[inline]
    fn from(inner: ExpectKind<T>) -> Self {
        Expects::from(Expect::Positive(inner))
    }
}

impl<T> From<&'static str> for Expects<T> {
    #[inline]
    fn from(msg: &'static str) -> Self {
        Expects::from(ExpectKind::Static(msg))
    }
}

impl<T> Expects<T> {
    /// Creates a new instance.
    #[inline]
    pub fn new(first: ExpectKind<T>) -> Self {
        Self::from(first)
    }

    /// Creates a new instance with [`Expect::Negative`].
    #[inline]
    pub fn new_neg(first: ExpectKind<T>) -> Self {
        Self::from(Expect::Negative(first))
    }

    /// Negates all elements.
    #[inline]
    pub fn negate(self) -> Self {
        self.map(Expect::negate)
    }

    /// Converts variant [`ExpectKind::Token`] of each elements.
    #[inline]
    pub fn map_tokens<F: FnMut(T) -> U, U>(self, mut f: F) -> Expects<U> {
        self.map(|e| e.map_token(&mut f))
    }
}
