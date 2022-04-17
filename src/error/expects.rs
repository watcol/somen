#[cfg(not(feature = "alloc"))]
mod no_std;
#[cfg(feature = "alloc")]
mod std;

#[cfg(feature = "alloc")]
pub use self::std::{Expect, Expects};
#[cfg(not(feature = "alloc"))]
pub use no_std::{Expect, Expects};

impl Expects {
    /// Creates a new instance.
    #[inline]
    pub fn new(first: Expect) -> Self {
        Self::from(first)
    }
}
