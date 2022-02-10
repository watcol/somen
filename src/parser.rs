//! Basic parsers and combinators.

pub mod streamed;

mod any;
mod cond;
mod either;
mod eof;
mod func;
mod future;
mod lazy;
mod map;
mod no_state;
mod opt;
mod or;
mod repeat;
mod token;
mod tuples;
mod value;

#[cfg(feature = "alloc")]
mod record;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
pub use any::Any;
pub use cond::Cond;
pub use either::Either;
pub use eof::Eof;
pub use func::Function;
pub use lazy::Lazy;
pub use map::{Expect, Map, MapErr, Spanned};
pub use no_state::NoState;
pub use opt::Opt;
pub use or::Or;
#[cfg(feature = "alloc")]
pub use record::{Record, WithRecord};
pub use repeat::{RangeArgument, Repeat};
pub use token::Token;
pub use value::Value;

use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{Expects, ParseResult};
#[cfg(feature = "alloc")]
use crate::stream::NoRewindInput;
use crate::stream::{Input, Positioned};
use future::ParseFuture;

/// Parses any token.
#[inline]
pub fn any<I: Positioned + ?Sized>() -> Any<I> {
    assert_parser(Any::new())
}

/// Successes if the input reached the end.
#[inline]
pub fn eof<I: Positioned + ?Sized>() -> Eof<I> {
    assert_parser(Eof::new())
}

/// Parses a token matches the condition.
#[inline]
pub fn is<I, F>(cond: F) -> Cond<I, F>
where
    I: Positioned + ?Sized,
    F: FnMut(&I::Ok) -> bool,
{
    assert_parser(Cond::new(cond))
}

/// Parses a token.
#[inline]
pub fn token<I>(token: I::Ok) -> Token<I, I::Ok>
where
    I: Positioned + ?Sized,
    I::Ok: Clone + Eq,
{
    assert_parser(Token::new(token))
}

/// Wrapping the function into a parser.
#[inline]
pub fn function<F, I, O, E, C>(f: F) -> Function<F, I, C>
where
    F: FnMut(Pin<&mut I>, &mut Context<'_>, &mut C) -> Poll<ParseResult<O, I>>,
    I: Positioned + ?Sized,
    C: Default,
{
    assert_parser(Function::new(f))
}

/// Produces the parser at the time of parsing.
#[inline]
pub fn lazy<F, P, I>(f: F) -> Lazy<F>
where
    F: FnMut() -> P,
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    assert_parser(Lazy::new(f))
}

/// Produces a value without parsing any tokens.
#[inline]
pub fn value<I: Positioned + ?Sized, T: Clone>(value: T) -> Value<I, T> {
    Value::new(value)
}

/// A trait for parsers.
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
    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self::Output, I>>;
}

/// An extension trait for [`Parser`].
pub trait ParserExt<I: Positioned + ?Sized>: Parser<I> {
    /// An asynchronous version of [`poll_parse`], which returns a [`Future`].
    ///
    /// [`poll_parse`]: Parser::poll_parse
    /// [`Future`]: core::future::Future
    #[inline]
    fn parse<'a, 'b>(&'a mut self, input: &'b mut I) -> ParseFuture<'a, 'b, Self, I, Self::State>
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

    /// Merges [`State`] into parser itself.
    ///
    /// [`State`]: Parser::State
    fn no_state(self) -> NoState<Self, Self::State>
    where
        Self: Sized,
    {
        assert_parser(NoState::new(self))
    }

    /// Wraps the parser into a [`Either`] to merge multiple types of parsers.
    fn left<R>(self) -> Either<Self, R>
    where
        Self: Sized,
        R: Parser<I, Output = Self::Output>,
    {
        assert_parser(Either::Left(self))
    }

    /// Wraps the parser into a [`Either`] to merge multiple types of parsers.
    fn right<L>(self) -> Either<L, Self>
    where
        Self: Sized,
        L: Parser<I, Output = Self::Output>,
    {
        assert_parser(Either::Right(self))
    }

    /// Returns consumed items instead of an output.
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    #[inline]
    fn record(self) -> Record<Self>
    where
        Self: Sized,
        I: NoRewindInput,
    {
        assert_parser(Record::new(self))
    }

    /// Returns an output beside consumed items.
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
    #[inline]
    fn with_record(self) -> WithRecord<Self>
    where
        Self: Sized,
        I: NoRewindInput,
    {
        assert_parser(WithRecord::new(self))
    }

    /// Parses with `self`, and then with `p`.
    fn and<P>(self, p: P) -> (Self, P)
    where
        Self: Sized,
        P: Parser<I>,
    {
        assert_parser((self, p))
    }

    /// Trying another parser if the parser is failed.
    #[inline]
    fn or<P>(self, other: P) -> Or<Self, P>
    where
        Self: Sized,
        I: Input,
        P: Parser<I, Output = Self::Output>,
    {
        assert_parser(Or::new(self, other))
    }

    /// Returns [`Some`] when parsing was successed.
    #[inline]
    fn opt(self) -> Opt<Self>
    where
        Self: Sized,
        I: Input,
    {
        assert_parser(Opt::new(self))
    }

    /// Returns a [`StreamedParser`] by repeating the parser while successing.
    ///
    /// [`StreamedParser`]: streamed::StreamedParser
    #[inline]
    fn repeat<R: RangeArgument>(self, range: R) -> Repeat<Self, R::Target>
    where
        Self: Sized,
        I: Input,
    {
        assert_streamed_parser(Repeat::new(self, range))
    }

    /// Converting an output value into another type.
    #[inline]
    fn map<F, O>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> O,
    {
        assert_parser(Map::new(self, f))
    }

    /// Modifying values expected by the parser.
    ///
    /// It is more easy to use [`expect`] when you just want to override the message by a
    /// `&'static str`.
    ///
    /// [`expect`]: Self::expect
    #[inline]
    fn map_err<F, O>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
        F: FnMut(Expects<I::Ok>) -> Expects<I::Ok>,
    {
        assert_parser(MapErr::new(self, f))
    }

    /// Overriding expected values by `&'static str`.
    ///
    /// [`map_err`] can be used if you want more complicated operation to expected values.
    ///
    /// [`map_err`]: Self::map_err
    #[inline]
    fn expect<O>(self, message: &'static str) -> Expect<Self>
    where
        Self: Sized,
    {
        assert_parser(Expect::new(self, message))
    }

    /// Overriding the position for error reportings by the span from the start of the parser to the
    /// end of it.
    #[inline]
    fn spanned(self) -> Spanned<Self>
    where
        Self: Sized,
    {
        assert_parser(Spanned::new(self))
    }
}

impl<P: Parser<I>, I: Positioned + ?Sized> ParserExt<I> for P {}

impl<'a, P: Parser<I> + ?Sized, I: Positioned + ?Sized> Parser<I> for &'a mut P {
    type Output = P::Output;
    type State = P::State;

    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self::Output, I>> {
        (**self).poll_parse(input, cx, state)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
impl<P: Parser<I> + ?Sized, I: Positioned + ?Sized> Parser<I> for Box<P> {
    type Output = P::Output;
    type State = P::State;

    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self::Output, I>> {
        (**self).poll_parse(input, cx, state)
    }
}

#[inline]
fn assert_parser<P: Parser<I>, I: Positioned + ?Sized>(parser: P) -> P {
    parser
}

use streamed::StreamedParser;
#[inline]
fn assert_streamed_parser<P: StreamedParser<I>, I: Positioned + ?Sized>(parser: P) -> P {
    parser
}
