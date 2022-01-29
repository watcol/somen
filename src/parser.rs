//! Basic parsers and combinators.

pub mod streamed;

mod any;
mod func;
mod future;
mod map;
mod opt;
mod repeat;
mod tuples;

#[cfg(feature = "alloc")]
mod record;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
pub use any::{Any, AnyError};
pub use func::Function;
pub use map::Map;
pub use opt::Opt;
#[cfg(feature = "alloc")]
pub use record::{Record, WithRecord};
pub use repeat::{RangeArgument, Repeat, RepeatError};

use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{ParseError, ParseResult};
#[cfg(feature = "alloc")]
use crate::stream::NoRewindInput;
use crate::stream::{Input, Positioned};
use future::ParseFuture;

/// Parses any token.
#[inline]
pub fn any<I: Positioned + ?Sized>() -> Any<I> {
    Any::new()
}

/// A parser calling a function.
pub fn function<F, I, O, E, C>(f: F) -> Function<F, I, C>
where
    F: Fn(
        Pin<&mut I>,
        &mut Context<'_>,
        &mut C,
    ) -> Poll<Result<O, ParseError<E, I::Error, I::Locator>>>,
    I: Positioned + ?Sized,
    C: Default,
{
    Function::new(f)
}

/// A trait for parsers.
pub trait Parser<I: Positioned + ?Sized> {
    /// The output type for the parser.
    type Output;

    /// The type of errors generated from the parser.
    type Error;

    /// The internal state used in [`poll_parse`].
    ///
    /// This state will be initialized by [`Default`] trait and stored in the [`Future`] object.
    ///
    /// [`poll_parse`]: Self::poll_parse
    /// [`Future`]: core::future::Future
    type State: Default;

    /// Parses the `input`, give an output.
    fn poll_parse(
        &self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>>;

    /// An asynchronous version of [`poll_parse`], which returns a [`Future`].
    ///
    /// [`poll_parse`]: Self::poll_parse
    /// [`Future`]: core::future::Future
    #[inline]
    fn parse<'a, 'b>(&'a self, input: &'b mut I) -> ParseFuture<'a, 'b, Self, I, Self::State>
    where
        I: Unpin,
    {
        ParseFuture::new(self, input)
    }

    /// Returns consumed items instead of an output.
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    #[inline]
    fn record(self) -> Record<Self>
    where
        I: NoRewindInput,
        Self: Sized,
    {
        Record::new(self)
    }

    /// Returns an output beside consumed items.
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    #[inline]
    fn with_record(self) -> WithRecord<Self>
    where
        I: NoRewindInput,
        Self: Sized,
    {
        WithRecord::new(self)
    }

    /// Converting an output value into another type.
    #[inline]
    fn map<F, O>(self, f: F) -> Map<Self, F>
    where
        I: Input,
        F: Fn(Self::Output) -> O,
        Self: Sized,
    {
        Map::new(self, f)
    }

    /// Returns [`Some`] when parsing was successed.
    #[inline]
    fn opt(self) -> Opt<Self>
    where
        I: Input,
        Self: Sized,
    {
        Opt::new(self)
    }

    /// Returns a [`StreamedParser`] by repeating the parser while successing.
    ///
    /// [`StreamedParser`]: streamed::StreamedParser
    #[inline]
    fn repeat<R: RangeArgument>(self, range: R) -> Repeat<Self, R::Target>
    where
        I: Input,
        Self: Sized,
    {
        Repeat::new(self, range)
    }
}

impl<'a, P: Parser<I> + ?Sized, I: Positioned + ?Sized> Parser<I> for &'a P {
    type Output = P::Output;
    type Error = P::Error;
    type State = P::State;

    fn poll_parse(
        &self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        (**self).poll_parse(input, cx, state)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<'a, P: Parser<I>, I: Positioned + ?Sized> Parser<I> for Box<P> {
    type Output = P::Output;
    type Error = P::Error;
    type State = P::State;

    fn poll_parse(
        &self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        (**self).poll_parse(input, cx, state)
    }
}
