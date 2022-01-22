//! Basic parsers and combinators.

pub mod streamed;

#[cfg(feature = "alloc")]
mod boxed;

#[cfg(feature = "alloc")]
pub use boxed::{BoxParser, ErrorBoxed, FutureBoxed};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::future::Future;

use crate::error::ParseResult;
use crate::stream::position::Positioned;

/// A trait for parsers.
///
/// ### Note
/// Lifetime `'parser` and `'input` are required when `Self::Future` borrows `self` or `input`.
///
/// This design will be changed when GATs([RFC](https://github.com/rust-lang/rfcs/pull/1598))
/// are stabilized, like this:
/// ```rust,ignored
/// pub trait Parser<I: ...> {
///     type Output;
///     type Error;
///     type Future<'a, 'b>: Future<Output = ...>;
///
///     fn parse<'a, 'b>(&'a self, &'b mut input) -> Self::Future<'a, 'b>;
///
///     // ...
/// }
/// ```
pub trait Parser<'parser, 'input, I: Positioned + ?Sized> {
    type Output;
    type Error;
    type Future: Future<Output = ParseResult<Self::Output, Self::Error, I::Error, I::Locator>>;

    /// Parses the `input`, returns a [`Future`].
    fn parse(&'parser self, input: &'input mut I) -> Self::Future;

    /// Wrapping the parser in a [`Box`].
    #[cfg(feature = "alloc")]
    #[inline]
    fn boxed<'a>(self) -> BoxParser<'a, 'parser, 'input, I, Self::Output, Self::Error, Self::Future>
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

    /// Wrapping errors in a [`Box`].
    #[cfg(feature = "alloc")]
    #[inline]
    fn box_error(self) -> ErrorBoxed<Self>
    where
        Self: Sized,
        Self::Error: core::fmt::Display + 'static,
    {
        ErrorBoxed::new(self)
    }
}
