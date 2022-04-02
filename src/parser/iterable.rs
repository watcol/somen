//! Tools for parsers return multiple outputs.

pub mod combinator;
pub mod flat;
pub mod generator;

mod stream;

use core::ops::RangeBounds;
use core::pin::Pin;
use core::task::Context;

use super::{
    assert_parser, ChoiceIterableParser, Either, Map, NoState, Opt, Or, Parser, Prefix, Skip,
    TryMap,
};
use crate::error::{Expects, PolledResult};
use crate::stream::{Input, Positioned};
use combinator::*;
use flat::*;
use stream::IterableParserStream;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// A conventional function to produce a nested [`or`] parser from a tuple of iterable parsers.
///
/// For example, `choice_iterable((a, b, c))` is equivalent to `a.or(b).or(c)`.
///
/// [`or`]: IterableParserExt::or
#[inline]
pub fn choice_iterable<C, I>(choice: C) -> C::IterableParser
where
    C: ChoiceIterableParser<I>,
    I: Positioned + ?Sized,
{
    choice.into_iterable_parser()
}

/// A trait for parsers return multiple outputs with [`TryStream`].
///
/// [`TryStream`]: futures_core::TryStream
#[cfg_attr(feature = "nightly", doc(notable_trait))]
pub trait IterableParser<I: Positioned + ?Sized> {
    /// The type for items of input stream.
    type Item;

    /// The internal state used in [`poll_parse_next`].
    ///
    /// This state will be initialized by [`Default`] trait and stored in the [`TryStream`].
    ///
    /// [`poll_parse_next`]: Self::poll_parse_next
    /// [`TryStream`]: futures_core::TryStream
    type State: Default;

    /// Takes an input, returns a [`Status`] of next output or [`None`].
    ///
    /// [`Status`]: crate::error::Status
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I>;

    /// The estimated size of returned stream.
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub trait IterableParserExt<I: Positioned + ?Sized>: IterableParser<I> {
    /// Returns a [`TryStream`] by invoking [`poll_parse_next`].
    ///
    /// [`TryStream`]: futures_core::TryStream
    /// [`poll_parse_next`]: IterableParser::poll_parse_next
    #[inline]
    fn parse_iterable<'a, 'b>(
        &'a mut self,
        input: &'b mut I,
    ) -> IterableParserStream<'a, 'b, Self, I, Self::State>
    where
        I: Unpin,
    {
        IterableParserStream::new(self, input)
    }

    /// Wraps the parser into a [`Box`].
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    #[inline]
    fn boxed<'a>(self) -> Box<dyn IterableParser<I, Item = Self::Item, State = Self::State> + 'a>
    where
        Self: Sized + 'a,
    {
        assert_iterable_parser(Box::new(self))
    }

    /// Merges [`State`] into the parser itself.
    ///
    /// [`State`]: IterableParser::State
    #[inline]
    fn no_state(self) -> NoState<Self, Self::State>
    where
        Self: Sized,
    {
        assert_iterable_parser(NoState::new(self))
    }

    /// Wraps the parser into a [`Either`] to merge multiple types of parsers.
    #[inline]
    fn left<R>(self) -> Either<Self, R>
    where
        Self: Sized,
        R: IterableParser<I, Item = Self::Item>,
    {
        assert_iterable_parser(Either::Left(self))
    }

    /// Wraps the parser into a [`Either`] to merge multiple types of parsers.
    #[inline]
    fn right<L>(self) -> Either<L, Self>
    where
        Self: Sized,
        L: IterableParser<I, Item = Self::Item>,
    {
        assert_iterable_parser(Either::Right(self))
    }

    /// Chains two iterable parsers and parses items in sequence.
    #[inline]
    fn chain<P>(self, p: P) -> (Self, P)
    where
        Self: Sized,
        P: IterableParser<I, Item = Self::Item>,
    {
        assert_iterable_parser((self, p))
    }

    /// Tries another parser if the first parser failed parsing.
    #[inline]
    fn or<P>(self, p: P) -> Or<Self, P>
    where
        I: Input,
        Self: Sized,
        P: IterableParser<I, Item = Self::Item>,
    {
        assert_iterable_parser(Or::new(self, p))
    }

    /// Returns [`Some`] if parsing is succeeded.
    #[inline]
    fn opt(self) -> Opt<Self>
    where
        Self: Sized,
        I: Input,
    {
        assert_iterable_parser(Opt::new(self))
    }

    /// Parses with `self`, then skips `p`.
    #[inline]
    fn skip<P>(self, p: P) -> Skip<Self, P>
    where
        Self: Sized,
        P: Parser<I>,
    {
        assert_iterable_parser(Skip::new(self, p))
    }

    /// Parses with `self` between `left` and `right`.
    #[inline]
    fn between<L, R>(self, left: L, right: R) -> Skip<Prefix<L, Self>, R>
    where
        Self: Sized,
        L: Parser<I>,
        R: Parser<I>,
    {
        assert_iterable_parser(Skip::new(Prefix::new(left, self), right))
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

    /// Consumes all outputs, returns the first element.
    ///
    /// This method is equivalent to `self.nth(0)`, and if the stream is empty, it returns `None`.
    #[inline]
    fn first(self) -> Nth<Self>
    where
        Self: Sized,
    {
        assert_parser(Nth::new(self, 0))
    }

    /// Consumes all outputs, returns the last element.
    ///
    /// If the stream is empty, it returns `None`.
    #[inline]
    fn last(self) -> Last<Self>
    where
        Self: Sized,
    {
        assert_parser(Last::new(self))
    }

    /// Consumes all outputs, returns the `n`th element.
    ///
    /// Note that `n` starts from `0` and if the length of stream less than `n`, it returns `None`.
    #[inline]
    fn nth(self, n: usize) -> Nth<Self>
    where
        Self: Sized,
    {
        assert_parser(Nth::new(self, n))
    }

    /// Consumes all outputs, returns `N` elements from index `start`.
    ///
    /// This method is equivalent to `self.indexes([start, start+1, ... , start+N-1])`.
    ///
    /// [`Parser`]: super::Parser
    #[inline]
    fn fill<const N: usize>(self, start: usize) -> Indexes<Self, N>
    where
        Self: Sized,
    {
        assert_parser(Indexes::new_fill(self, start))
    }

    /// Consumes all outputs, returns multiple elements specified by `ns`, an ascending ordered array of
    /// indexes.
    ///
    /// Note that `n` starts from `0` and if the length of stream less than `n`, it returns `None`.
    ///
    /// # Panics
    /// if `ns` is not ascending ordered.
    #[inline]
    fn indexes<const N: usize>(self, ns: [usize; N]) -> Indexes<Self, N>
    where
        Self: Sized,
    {
        assert_parser(Indexes::new(self, ns))
    }

    /// Repeats the iterable parser like [`ParserExt::repeat`], and flattens into one iterable
    /// parser.
    ///
    /// [`ParserExt::repeat`]: super::ParserExt::repeat
    #[inline]
    fn flat_repeat<R>(self, range: R) -> FlatRepeat<Self, R>
    where
        Self: Sized,
        I: Input,
        R: RangeBounds<usize>,
    {
        assert_iterable_parser(FlatRepeat::new(self, range))
    }

    /// Repeats the iterable parser like [`ParserExt::times`], and flattens into one iterable
    /// parser.
    ///
    /// [`ParserExt::times`]: super::ParserExt::times
    #[inline]
    fn flat_times(self, n: usize) -> FlatTimes<Self>
    where
        Self: Sized,
    {
        assert_iterable_parser(FlatTimes::new(self, n))
    }

    /// Repeats the iterable parser with separaters like [`ParserExt::sep_by`], and flattens into one
    /// iterable parser.
    ///
    /// [`ParserExt::sep_by`]: super::ParserExt::sep_by
    #[inline]
    fn flat_sep_by<P, R>(self, sep: P, range: R) -> FlatSepBy<Self, P, R>
    where
        Self: Sized,
        I: Input,
        P: Parser<I>,
        R: RangeBounds<usize>,
    {
        assert_iterable_parser(FlatSepBy::new(self, sep, range))
    }

    /// Repeats the iterable parser with separaters like [`ParserExt::sep_by_end`], and flattens
    /// into one iterable parser.
    ///
    /// [`ParserExt::sep_by_end`]: super::ParserExt::sep_by_end
    #[inline]
    fn flat_sep_by_end<P, R>(self, sep: P, range: R) -> FlatSepByEnd<Self, P, R>
    where
        Self: Sized,
        I: Input,
        P: Parser<I>,
        R: RangeBounds<usize>,
    {
        assert_iterable_parser(FlatSepByEnd::new(self, sep, range))
    }

    /// Repeats the iterable parser with separaters like [`ParserExt::sep_by_times`], and flattens
    /// into one iterable parser.
    ///
    /// [`ParserExt::sep_by_times`]: super::ParserExt::sep_by_times
    #[inline]
    fn flat_sep_by_times<P>(self, sep: P, count: usize) -> FlatSepByTimes<Self, P>
    where
        Self: Sized,
        P: Parser<I>,
    {
        assert_iterable_parser(FlatSepByTimes::new(self, sep, count))
    }

    /// Repeats the iterable parser with separaters like [`ParserExt::sep_by_end_times`], and
    /// flattens into one iterable parser.
    ///
    /// [`ParserExt::sep_by_end_times`]: super::ParserExt::sep_by_end_times
    #[inline]
    fn flat_sep_by_end_times<P>(self, sep: P, count: usize) -> FlatSepByEndTimes<Self, P>
    where
        Self: Sized,
        I: Input,
        P: Parser<I>,
    {
        assert_iterable_parser(FlatSepByEndTimes::new(self, sep, count))
    }

    /// Repeats the iterable parser until `end` like [`ParserExt::until`], and flattens into one
    /// iterable parser.
    ///
    /// [`ParserExt::until`]: super::ParserExt::until
    #[inline]
    fn flat_until<P>(self, end: P) -> FlatUntil<Self, P>
    where
        Self: Sized,
        I: Input,
        P: Parser<I>,
    {
        assert_iterable_parser(FlatUntil::new(self, end))
    }

    /// Converts an output value into another type.
    #[inline]
    fn map<F, O>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> O,
    {
        assert_iterable_parser(Map::new(self, f))
    }

    /// Converts an output value into another type.
    #[inline]
    fn try_map<F, O, E>(self, f: F) -> TryMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Result<O, E>,
        E: Into<Expects<I::Ok>>,
    {
        assert_iterable_parser(TryMap::new(self, f))
    }

    /// Returns current iteration count with elements.
    ///
    /// The returned parser's item will be `(usize, Self::Item)`.
    #[inline]
    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        assert_iterable_parser(Enumerate::new(self))
    }

    /// Flattens iteratable items.
    #[inline]
    fn flatten(self) -> Flatten<Self>
    where
        Self: Sized,
        Self::Item: IntoIterator,
    {
        assert_iterable_parser(Flatten::new(self))
    }

    /// Only returns items matches the condition.
    #[inline]
    fn filter<F>(self, f: F) -> Filter<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool,
    {
        assert_iterable_parser(Filter::new(self, f))
    }

    /// Folds items into an accumulator by repeatedly applying a function.
    #[inline]
    fn fold<Q, F>(self, init: Q, f: F) -> Fold<Self, Q, F>
    where
        Self: Sized,
        Q: Parser<I>,
        F: FnMut(Q::Output, Self::Item) -> Q::Output,
    {
        assert_parser(Fold::new(self, init, f))
    }

    /// Tries to fold items into an accumulator by repeatedly applying a failable function.
    #[inline]
    fn try_fold<Q, F, E>(self, init: Q, f: F) -> TryFold<Self, Q, F>
    where
        Self: Sized,
        Q: Parser<I>,
        F: FnMut(Q::Output, Self::Item) -> Result<Q::Output, E>,
        E: Into<Expects<I::Ok>>,
    {
        assert_parser(TryFold::new(self, init, f))
    }

    /// Reduces items into a item by repeatedly applying a function.
    #[inline]
    fn reduce<F>(self, f: F) -> Reduce<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item, Self::Item) -> Self::Item,
    {
        assert_parser(Reduce::new(self, f))
    }

    /// Tries to reduce items into a item by repeatedly applying a failable function.
    #[inline]
    fn try_reduce<F, E>(self, f: F) -> TryReduce<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item, Self::Item) -> Result<Self::Item, E>,
        E: Into<Expects<I::Ok>>,
    {
        assert_parser(TryReduce::new(self, f))
    }

    /// Holds internal state, applies the function item by item and returns the output (if it is
    /// [`Some`]).
    #[inline]
    fn scan<Q, F, T>(self, init: Q, f: F) -> Scan<Self, Q, F>
    where
        Self: Sized,
        Q: Parser<I>,
        F: FnMut(&mut Q::Output, Self::Item) -> Option<T>,
    {
        assert_iterable_parser(Scan::new(self, init, f))
    }

    /// Holds internal state, applies the failable function item by item and tries to return the
    /// output (if it is [`Some`]).
    #[inline]
    fn try_scan<Q, F, T, E>(self, init: Q, f: F) -> TryScan<Self, Q, F>
    where
        Self: Sized,
        Q: Parser<I>,
        F: FnMut(&mut Q::Output, Self::Item) -> Result<Option<T>, E>,
        E: Into<Expects<I::Ok>>,
    {
        assert_iterable_parser(TryScan::new(self, init, f))
    }
}

impl<P: IterableParser<I> + ?Sized, I: Positioned + ?Sized> IterableParserExt<I> for P {}

impl<'a, P: IterableParser<I> + ?Sized, I: Positioned + ?Sized> IterableParser<I> for &'a mut P {
    type Item = P::Item;
    type State = P::State;

    #[inline]
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        (**self).poll_parse_next(input, cx, state)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<P: IterableParser<I> + ?Sized, I: Positioned + ?Sized> IterableParser<I> for Box<P> {
    type Item = P::Item;
    type State = P::State;

    #[inline]
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        (**self).poll_parse_next(input, cx, state)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

#[inline]
pub(super) fn assert_iterable_parser<P: IterableParser<I>, I: Positioned + ?Sized>(parser: P) -> P {
    parser
}
