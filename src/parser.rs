//! Basic parsers and combinators.

pub mod streamed;

mod future;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::pin::Pin;
use core::task::Context;

use crate::error::{PolledResult, Tracker};
#[cfg(feature = "alloc")]
use crate::stream::Positioned;
use future::ParseFuture;

/// A trait for parsers.
#[cfg_attr(feature = "nightly", doc(notable_trait))]
pub trait Parser<I: Positioned + ?Sized> {
    /// The output type for the parser.
    type Output;

    /// The internal state used in [`poll_parse`].
    ///
    /// This state will be initialized by [`Default`] trait and stored in the [`Future`] object.
    ///
    /// [`poll_parse`]: Self::poll_parse
    /// [`Future`]: core::future::Future
    type State: Default;

    /// Parses the `input`, give an output.
    ///
    /// `tracker` is for tracking ignored errors. When an error has occured, tracked errors will be
    /// merged to the original error. Tracked errors are valid only until a new token is consumed,
    /// so you must invoke `tracker.clear` beside `input.try_poll_next`.
    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I>;
}

/// An extension trait for [`Parser`].
pub trait ParserExt<I: Positioned + ?Sized>: Parser<I> {
    /// An asynchronous version of [`poll_parse`], which returns a [`Future`].
    ///
    /// [`poll_parse`]: Parser::poll_parse
    /// [`Future`]: core::future::Future
    #[inline]
    fn parse<'a, 'b>(
        &'a mut self,
        input: &'b mut I,
    ) -> ParseFuture<'a, 'b, Self, I, Self::State, I::Ok>
    where
        I: Unpin,
    {
        ParseFuture::new(self, input)
    }

    /// Wraps the parser into a [`Box`].
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    #[inline]
    fn boxed<'a>(self) -> Box<dyn Parser<I, Output = Self::Output, State = Self::State> + 'a>
    where
        Self: Sized + 'a,
    {
        assert_parser(Box::new(self))
    }
}

impl<P: Parser<I>, I: Positioned + ?Sized> ParserExt<I> for P {}

impl<'a, P: Parser<I> + ?Sized, I: Positioned + ?Sized> Parser<I> for &'a mut P {
    type Output = P::Output;
    type State = P::State;

    #[inline]
    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        (**self).poll_parse(input, cx, state, tracker)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<P: Parser<I> + ?Sized, I: Positioned + ?Sized> Parser<I> for Box<P> {
    type Output = P::Output;
    type State = P::State;

    #[inline]
    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        (**self).poll_parse(input, cx, state, tracker)
    }
}

#[inline]
fn assert_parser<P: Parser<I>, I: Positioned + ?Sized>(parser: P) -> P {
    parser
}
