//! Basic parsers and combinators.

mod future;
use future::ParseFuture;

use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::stream::TryStream;

use crate::error::{ParseError, ParseResult};
use crate::stream::position::Positioned;

/// A trait for parsers.
pub trait Parser {
    /// The type of input items.
    type Item;
    /// The type of the output value.
    type Output;
    /// The type of errors while parsing.
    type Error: ParseError;

    /// Parse the `stream`, get an output.
    fn poll_parse<S>(
        &self,
        stream: Pin<&mut S>,
        cx: &mut Context<'_>,
    ) -> Poll<ParseResult<Self, S>>
    where
        Self: Sized,
        S: TryStream<Ok = Self::Item>
            + Positioned<Position = <Self::Error as ParseError>::Position>
            + ?Sized;

    /// An asynchronous version of [`poll_parse`], which returns a [`Future`] object.
    ///
    /// [`poll_parse`]: Self::poll_parse
    /// [`Future`]: core::future::Future
    fn parse<S>(&self, stream: &mut S) -> ParseFuture<'_, '_, Self, S>
    where
        Self: Sized;
}
