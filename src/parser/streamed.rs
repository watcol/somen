//! Tools for parsers return multiple outputs.

mod stream;

use core::pin::Pin;
use core::task::Context;

use crate::error::{PolledResult, Tracker};
use crate::stream::Positioned;
use stream::ParserStream;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// A trait for parsers return multiple outputs with [`TryStream`].
///
/// [`TryStream`]: futures_core::TryStream
#[cfg_attr(feature = "nightly", doc(notable_trait))]
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
    ) -> PolledResult<Option<Self::Item>, I>;

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
    ) -> PolledResult<Option<Self::Item>, I> {
        (**self).poll_parse_next(input, cx, state, tracker)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
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
    ) -> PolledResult<Option<Self::Item>, I> {
        (**self).poll_parse_next(input, cx, state, tracker)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

#[inline]
pub(super) fn assert_streamed_parser<P: StreamedParser<I>, I: Positioned + ?Sized>(parser: P) -> P {
    parser
}
