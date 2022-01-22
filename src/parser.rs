//! Basic parsers and combinators.

pub mod streamed;

#[cfg(feature = "alloc")]
mod boxed;

#[cfg(feature = "alloc")]
pub use boxed::{BoxParser, FutureBoxed};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::future::Future;

use crate::error::ParseResult;
use crate::stream::position::Positioned;

/// A trait for parsers.
pub trait Parser<I: Positioned + ?Sized> {
    type Output;
    type Error;
    type Future: Future<Output = ParseResult<Self, I>>;

    /// Parses the `input`, returns a [`Future`].
    fn parse(&self, input: &mut I) -> Self::Future;

    /// Wrapping the parser in a [`Box`].
    #[cfg(feature = "alloc")]
    #[inline]
    fn boxed<'a>(self) -> BoxParser<'a, I, Self::Output, Self::Error, Self::Future>
    where
        Self: Sized + 'a,
    {
        Box::new(self)
    }

    /// Wrapping returned futures in a [`Box`].
    #[cfg(feature = "alloc")]
    #[inline]
    fn box_future<'a>(self) -> FutureBoxed<'a, Self>
    where
        Self: Sized,
        Self::Future: Sized + 'a,
    {
        FutureBoxed::new(self)
    }
}
