//! Basic parsers and combinators.

pub mod streamed;

mod any;
mod cond;
mod eof;
mod opt;
mod peek;
mod repeat;
mod set;
mod token;
mod tuples;
mod value;

mod future;
mod utils;

pub use any::Any;
pub use cond::{Is, IsNot, IsSome};
pub use eof::Eof;
pub use opt::Opt;
pub use peek::{Fail, Peek};
pub use repeat::Repeat;
pub use set::{NoneOf, OneOf, Set};
pub use token::{Not, Token};
pub use value::{Value, ValueFn};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::ops::RangeBounds;
use core::pin::Pin;
use core::task::Context;

use crate::error::PolledResult;
use crate::stream::{Input, Positioned};
use future::ParseFuture;
use streamed::assert_streamed_parser;

/// Parses any token.
#[inline]
pub fn any<I: Positioned + ?Sized>() -> Any<I> {
    assert_parser(Any::new())
}

/// Succeeds if the input reached the end.
#[inline]
pub fn eof<I: Positioned + ?Sized>() -> Eof<I> {
    assert_parser(Eof::new())
}

/// Produces a value without parsing any tokens.
#[inline]
pub fn value<I: Positioned + ?Sized, T: Clone>(value: T) -> Value<I, T> {
    assert_parser(Value::new(value))
}

/// Produces a value by the function without parsing any tokens.
#[inline]
pub fn value_fn<I: Positioned + ?Sized, F: FnMut() -> T, T>(f: F) -> ValueFn<I, F> {
    assert_parser(ValueFn::new(f))
}

/// Parses a token.
#[inline]
pub fn token<I>(token: I::Ok) -> Token<I, I::Ok>
where
    I: Positioned + ?Sized,
    I::Ok: Clone + PartialEq,
{
    assert_parser(Token::new(token))
}

/// Parses any token except `token`.
#[inline]
pub fn not<I>(token: I::Ok) -> Not<I, I::Ok>
where
    I: Positioned + ?Sized,
    I::Ok: Clone + PartialEq,
{
    assert_parser(Not::new(token))
}

/// Succeeds if a parsed token matches one of the set.
#[inline]
pub fn one_of<I, S>(set: S) -> OneOf<I, S>
where
    I: Positioned + ?Sized,
    S: Set<I::Ok>,
{
    assert_parser(OneOf::new(set))
}

/// Succeeds if a parsed token doesn't match one of the set.
#[inline]
pub fn none_of<I, S>(set: S) -> NoneOf<I, S>
where
    I: Positioned + ?Sized,
    S: Set<I::Ok>,
{
    assert_parser(NoneOf::new(set))
}

/// Parses a token matches the condition.
#[inline]
pub fn is<I, F>(cond: F) -> Is<I, F>
where
    I: Positioned + ?Sized,
    F: FnMut(&I::Ok) -> bool,
{
    assert_parser(Is::new(cond))
}

/// Parses a token does not match the condition.
#[inline]
pub fn is_not<I, F>(cond: F) -> IsNot<I, F>
where
    I: Positioned + ?Sized,
    F: FnMut(&I::Ok) -> bool,
{
    assert_parser(IsNot::new(cond))
}

/// Parses a token, pass the token to the function and succeeds if the returned value is [`Some`].
#[inline]
pub fn is_some<I, F, O>(cond: F) -> IsSome<I, F>
where
    I: Positioned + ?Sized,
    F: FnMut(&I::Ok) -> Option<O>,
{
    assert_parser(IsSome::new(cond))
}

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
    ) -> PolledResult<Self::Output, I>;
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

    /// Returns a parse result without consuming input.
    #[inline]
    fn peek(self) -> Peek<Self>
    where
        Self: Sized,
        I: Input,
    {
        assert_parser(Peek::new(self))
    }

    /// Succeeds if the parser failed parsing. Never consumes input.
    #[inline]
    fn fail(self) -> Fail<Self>
    where
        Self: Sized,
        I: Input,
    {
        assert_parser(Fail::new(self))
    }

    /// Parses with `self`, and then with `p`.
    #[inline]
    fn and<P>(self, p: P) -> (Self, P)
    where
        Self: Sized,
        P: Parser<I>,
    {
        assert_parser((self, p))
    }

    /// Returns [`Some`] if parsing is succeeded.
    #[inline]
    fn opt(self) -> Opt<Self>
    where
        Self: Sized,
        I: Input,
    {
        assert_parser(Opt::new(self))
    }

    /// Returns a [`StreamedParser`] by repeating the parser while succeeding.
    ///
    /// [`StreamedParser`]: streamed::StreamedParser
    #[inline]
    fn repeat<R>(self, range: R) -> Repeat<Self, R>
    where
        Self: Sized,
        R: RangeBounds<usize>,
        I: Input,
    {
        assert_streamed_parser(Repeat::new(self, range))
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
    ) -> PolledResult<Self::Output, I> {
        (**self).poll_parse(input, cx, state)
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
    ) -> PolledResult<Self::Output, I> {
        (**self).poll_parse(input, cx, state)
    }
}

#[inline]
fn assert_parser<P: Parser<I>, I: Positioned + ?Sized>(parser: P) -> P {
    parser
}
