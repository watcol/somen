//! Tools for parsers return multiple outputs.

mod collect;
mod stream;

use core::pin::Pin;
use core::task::{Context, Poll};

pub use collect::Collect;

use super::{assert_parser, NoState};
use crate::error::{ParseResult, Tracker};
use crate::stream::position::Positioned;
use stream::ParserStream;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// A trait for parsers return multiple outputs with [`TryStream`].
///
/// [`TryStream`]: futures_core::TryStream
pub trait StreamedParser<I: Positioned + ?Sized> {
    /// The type for items of input stream.
    type Item;

    /// The internal state used in [`poll_parse_next`].
    ///
    /// This state will be initialized by [`Default`] trait and stored in the [`TryStream`].
    ///
    /// [`poll_parse_next`]: Self::poll_parse_next
    /// [`TryStream`]: futures_core::TryStream
    type State: Default;

    /// Takes an input, returns a next output or [`None`].
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>>;
}

pub trait StreamedParserExt<I: Positioned + ?Sized>: StreamedParser<I> {
    /// Returning a [`TryStream`] by invoking [`poll_parse_next`].
    ///
    /// [`TryStream`]: futures_core::TryStream
    /// [`poll_parse_next`]: StreamedParser::poll_parse_next
    #[inline]
    fn parse_streamed<'a, 'b>(
        &'a mut self,
        input: &'b mut I,
    ) -> ParserStream<'a, 'b, Self, I, Self::State, I::Ok>
    where
        I: Unpin,
    {
        ParserStream::new(self, input)
    }

    /// Wraps the parser into a [`Box`].
    #[cfg(feature = "alloc")]
    #[inline]
    fn boxed<'a>(self) -> Box<dyn StreamedParser<I, Item = Self::Item, State = Self::State> + 'a>
    where
        Self: Sized + 'a,
    {
        assert_streamed_parser(Box::new(self))
    }

    /// Merges [`State`] into the parser itself.
    ///
    /// [`State`]: StreamedParser::State
    #[cfg(feature = "alloc")]
    #[inline]
    fn no_state(self) -> NoState<Self, Self::State>
    where
        Self: Sized,
    {
        assert_streamed_parser(NoState::new(self))
    }

    /// Returns a [`Parser`] by collecting all the outputs.
    ///
    /// [`Parser`]: super::Parser
    #[inline]
    fn collect<E: Default + Extend<Self::Item>>(self) -> Collect<Self, E>
    where
        Self: Sized,
    {
        assert_parser(Collect::new(self))
    }
}

impl<P: StreamedParser<I> + ?Sized, I: Positioned + ?Sized> StreamedParserExt<I> for P {}

impl<'a, P: StreamedParser<I> + ?Sized, I: Positioned + ?Sized> StreamedParser<I> for &'a mut P {
    type Item = P::Item;
    type State = P::State;

    #[inline]
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        (**self).poll_parse_next(input, cx, state, tracker)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<P: StreamedParser<I> + ?Sized, I: Positioned + ?Sized> StreamedParser<I> for Box<P> {
    type Item = P::Item;
    type State = P::State;

    #[inline]
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        (**self).poll_parse_next(input, cx, state, tracker)
    }
}

#[inline]
pub(super) fn assert_streamed_parser<P: StreamedParser<I>, I: Positioned + ?Sized>(parser: P) -> P {
    parser
}
