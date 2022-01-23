//! Basic parsers and combinators.

pub mod streamed;

mod any;
mod future;
mod opt;

#[cfg(feature = "alloc")]
mod boxed;

pub use any::{Any, AnyError};
#[cfg(feature = "alloc")]
pub use boxed::{BoxError, BoxParser};
pub use opt::Opt;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::pin::Pin;
use core::task::{Context, Poll};

use future::ParseFuture;

use crate::error::ParseResult;
use crate::stream::{Input, Positioned};

/// Parses any token.
#[inline]
pub fn any() -> Any {
    Any::new()
}

/// A trait for parsers.
pub trait Parser<I: Positioned + ?Sized> {
    type Output;
    type Error;

    /// Parses the `input`, give an output.
    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
    ) -> Poll<ParseResult<Self, I>>;
}

/// An extension trait for [`Parser`].
pub trait ParserExt<I: Positioned + ?Sized>: Parser<I> + private::Sealed<I> {
    /// An asynchronous version of [`poll_parse`], which returns a [`Future`].
    ///
    /// [`poll_parse`]: self::Parser::poll_parse
    /// [`Future`]: core::future::Future
    #[inline]
    fn parse<'a, 'b>(&'a mut self, input: &'b mut I) -> ParseFuture<'a, 'b, Self, I> {
        ParseFuture::new(self, input)
    }

    fn opt(self) -> Opt<Self, I::Marker>
    where
        I: Input,
        Self: Sized,
    {
        Opt::new(self)
    }

    /// Wrapping the parser in a [`Box`].
    #[cfg(feature = "alloc")]
    #[inline]
    fn boxed<'a>(self) -> BoxParser<'a, I, Self::Output, Self::Error>
    where
        Self: Sized + 'a,
    {
        Box::new(self)
    }

    /// Wrapping errors in a [`Box`].
    #[cfg(feature = "alloc")]
    #[inline]
    fn box_error(self) -> BoxError<Self>
    where
        Self: Sized,
        Self::Error: core::fmt::Display + 'static,
    {
        BoxError::new(self)
    }
}

impl<P: Parser<I>, I: Positioned + ?Sized> ParserExt<I> for P {}

mod private {
    use super::Parser;
    use crate::stream::Positioned;

    pub trait Sealed<I: ?Sized> {}

    impl<P: Parser<I>, I: Positioned + ?Sized> Sealed<I> for P {}
}
