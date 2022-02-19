//! Tools for parsers return multiple outputs.

mod choice;
mod collect;
mod fill;
mod stream;
mod tuples;

use core::pin::Pin;
use core::task::{Context, Poll};

pub use choice::ChoiceStreamedParser;
pub use collect::{Collect, Count, Discard};
pub use fill::Fill;

use super::{assert_parser, AheadOf, Behind, Between, Either, Map, NoState, Or, Parser, TryMap};
use crate::error::{Expects, ParseResult, Tracker};
use crate::stream::{Input, Positioned};
use stream::ParserStream;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// A conventional function to produce [`or`] streamed parser from tuples.
///
/// For example, `choice_streamed((a, b, c))` is equivalent to `a.or(b).or(c)`.
///
/// [`or`]: StreamedParserExt::or
#[inline]
pub fn choice_streamed<C, I>(choice: C) -> C::Parser
where
    C: ChoiceStreamedParser<I>,
    I: Positioned + ?Sized,
{
    choice.into_streamed_parser()
}

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

    /// The estimated size of returned stream.
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
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

    /// Wraps the parser into a [`Either`] to merge multiple types of parsers.
    #[inline]
    fn left<R>(self) -> Either<Self, R>
    where
        Self: Sized,
        R: StreamedParser<I, Item = Self::Item>,
    {
        assert_streamed_parser(Either::Left(self))
    }

    /// Wraps the parser into a [`Either`] to merge multiple types of parsers.
    #[inline]
    fn right<L>(self) -> Either<L, Self>
    where
        Self: Sized,
        L: StreamedParser<I, Item = Self::Item>,
    {
        assert_streamed_parser(Either::Right(self))
    }

    /// Chains two streams and parses items in sequence.
    #[inline]
    fn chain<P>(self, p: P) -> (Self, P)
    where
        Self: Sized,
        P: StreamedParser<I, Item = Self::Item>,
    {
        assert_streamed_parser((self, p))
    }

    /// Trys another parser if the first parser failed parsing.
    #[inline]
    fn or<P>(self, p: P) -> Or<Self, P>
    where
        I: Input,
        Self: Sized,
        P: StreamedParser<I, Item = Self::Item>,
    {
        assert_streamed_parser(Or::new(self, p))
    }

    /// Parses with `self` ahead of `p`.
    #[inline]
    fn ahead_of<P>(self, p: P) -> AheadOf<Self, P>
    where
        Self: Sized,
        P: Parser<I>,
    {
        assert_streamed_parser(AheadOf::new(self, p))
    }

    /// Parses with `self` behind `p`.
    #[inline]
    fn behind<P>(self, p: P) -> Behind<Self, P>
    where
        Self: Sized,
        P: Parser<I>,
    {
        assert_streamed_parser(Behind::new(self, p))
    }

    /// Parses with `self` between `left` and `right`.
    #[inline]
    fn between<L, R>(self, left: L, right: R) -> Between<Self, L, R>
    where
        Self: Sized,
        L: Parser<I>,
        R: Parser<I>,
    {
        assert_streamed_parser(Between::new(self, left, right))
    }

    /// Returns a [`Parser`] parses all items and returns `()`.
    ///
    /// [`Parser`]: super::Parser
    #[inline]
    fn discard(self) -> Discard<Self>
    where
        Self: Sized,
    {
        assert_parser(Discard::new(self))
    }

    /// Returns a [`Parser`] outputs [`usize`] by couting up all items.
    ///
    /// [`Parser`]: super::Parser
    #[inline]
    fn count(self) -> Count<Self>
    where
        Self: Sized,
    {
        assert_parser(Count::new(self))
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

    /// Returns a [`Parser`] by collecting exact `N` items into an array.
    ///
    /// If the number of items is not `N`, it returns a fatal error.
    ///
    /// [`Parser`]: super::Parser
    #[inline]
    fn fill<const N: usize>(self) -> Fill<Self, N>
    where
        Self: Sized,
    {
        assert_parser(Fill::new(self))
    }

    /// Converting an output value into another type.
    #[inline]
    fn map<F, O>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> O,
    {
        assert_streamed_parser(Map::new(self, f))
    }

    /// Converting an output value into another type.
    #[inline]
    fn try_map<F, O, E>(self, f: F) -> TryMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Result<O, E>,
        E: Into<Expects<I::Ok>>,
    {
        assert_streamed_parser(TryMap::new(self, f))
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
