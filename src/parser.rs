//! Basic parsers and combinators.

pub mod streamed;

mod any;
mod choice;
mod cond;
mod either;
mod eof;
mod errors;
mod func;
mod future;
mod lazy;
mod map;
mod no_state;
mod opt;
mod or;
mod peek;
mod position;
mod repeat;
mod satisfy;
mod sep_by;
mod set;
mod skip;
mod then;
mod times;
mod token;
mod tokens;
mod tuples;
mod until;
mod value;

mod utils;

#[cfg(feature = "alloc")]
mod record;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
pub use any::Any;
pub use choice::ChoiceParser;
pub use cond::{Is, IsNot, IsSome};
pub use either::Either;
pub use eof::Eof;
pub use errors::{Expect, Fatal, MapErr, Spanned};
pub use func::Function;
pub use lazy::Lazy;
pub use map::{Map, TryMap};
pub use no_state::NoState;
pub use opt::Opt;
pub use or::Or;
pub use peek::{Fail, Peek};
pub use position::{Position, WithPosition};
#[cfg(feature = "alloc")]
pub use record::{Record, WithRecord};
pub use repeat::Repeat;
pub use satisfy::Satisfy;
pub use sep_by::{SepBy, SepByEnd};
pub use set::{NoneOf, OneOf, Set};
pub use skip::{AheadOf, Behind, Between, Discard};
pub use then::{Then, TryThen};
pub use times::Times;
pub use token::{Not, Token};
pub use tokens::Tokens;
pub use until::Until;
pub use value::{Value, ValueFn};

use core::ops::RangeBounds;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{Expects, ParseResult, Tracker};
#[cfg(feature = "alloc")]
use crate::stream::NoRewindInput;
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

/// Parses a sequence of tokens.
#[inline]
pub fn tokens<'a, I, T>(tokens: T) -> Tokens<'a, I, T>
where
    I: Positioned + ?Sized,
    I::Ok: PartialEq,
    T: IntoIterator<Item = &'a I::Ok> + Clone,
{
    assert_parser(Tokens::new(tokens))
}

/// Wrapping the function into a parser or a streaned parser.
#[inline]
pub fn function<F, I, O, E, C>(f: F) -> Function<F, I, C>
where
    F: FnMut(Pin<&mut I>, &mut Context<'_>, &mut C, &mut Tracker<I::Ok>) -> Poll<ParseResult<O, I>>,
    I: Positioned + ?Sized,
    C: Default,
{
    assert_parser(Function::new(f))
}

/// Produces the parser (or streamed parser) at the time of parsing.
#[inline]
pub fn lazy<F, P>(f: F) -> Lazy<F>
where
    F: FnMut() -> P,
{
    Lazy::new(f)
}

/// A conventional function to produce [`or`] parser from tuples.
///
/// For example, `choice((a, b, c))` is equivalent to `a.or(b).or(c)`.
///
/// [`or`]: ParserExt::or
#[inline]
pub fn choice<C, I>(choice: C) -> C::Parser
where
    C: ChoiceParser<I>,
    I: Input + ?Sized,
{
    assert_parser(choice.into_parser())
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

/// Returns the current position of input.
#[inline]
pub fn position<I: Positioned + ?Sized>() -> Position<I> {
    assert_parser(Position::new())
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
    ) -> Poll<ParseResult<Self::Output, I>>;
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

    /// Merges [`State`] into parser itself.
    ///
    /// [`State`]: Parser::State
    #[inline]
    fn no_state(self) -> NoState<Self, Self::State>
    where
        Self: Sized,
    {
        assert_parser(NoState::new(self))
    }

    /// Wraps the parser into a [`Either`] to merge multiple types of parsers.
    #[inline]
    fn left<R>(self) -> Either<Self, R>
    where
        Self: Sized,
        R: Parser<I, Output = Self::Output>,
    {
        assert_parser(Either::Left(self))
    }

    /// Wraps the parser into a [`Either`] to merge multiple types of parsers.
    #[inline]
    fn right<L>(self) -> Either<L, Self>
    where
        Self: Sized,
        L: Parser<I, Output = Self::Output>,
    {
        assert_parser(Either::Right(self))
    }

    /// Returns the position of parsed tokens with an output.
    #[inline]
    fn with_position(self) -> WithPosition<Self>
    where
        Self: Sized,
    {
        assert_parser(WithPosition::new(self))
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

    /// Trying another parser if the parser failed parsing.
    #[inline]
    fn or<P>(self, other: P) -> Or<Self, P>
    where
        Self: Sized,
        I: Input,
        P: Parser<I, Output = Self::Output>,
    {
        assert_parser(Or::new(self, other))
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

    /// Parses with `self` between `left` and `right`.
    #[inline]
    fn between<L, R>(self, left: L, right: R) -> Between<Self, L, R>
    where
        Self: Sized,
        L: Parser<I>,
        R: Parser<I>,
    {
        assert_parser(Between::new(self, left, right))
    }

    /// Parses with `self` ahead of `p`.
    #[inline]
    fn ahead_of<P>(self, p: P) -> AheadOf<Self, P>
    where
        Self: Sized,
        P: Parser<I>,
    {
        assert_parser(AheadOf::new(self, p))
    }

    /// Parses with `self` behind `p`.
    #[inline]
    fn behind<P>(self, p: P) -> Behind<Self, P>
    where
        Self: Sized,
        P: Parser<I>,
    {
        assert_parser(Behind::new(self, p))
    }

    /// Discarding the parse results.
    #[inline]
    fn discard(self) -> Discard<Self>
    where
        Self: Sized,
    {
        assert_parser(Discard::new(self))
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

    /// Returns [`Some`] if parsing is succeeded.
    #[inline]
    fn opt(self) -> Opt<Self>
    where
        Self: Sized,
        I: Input,
    {
        assert_parser(Opt::new(self))
    }

    /// Returns a [`StreamedParser`] by wrapping the parser to return output exactly once.
    ///
    /// This method is equivalent to `self.times(1)`.
    ///
    /// [`StreamedParser`]: streamed::StreamedParser
    #[inline]
    fn once(self) -> Times<Self>
    where
        Self: Sized,
        I: Positioned,
    {
        assert_streamed_parser(Times::new(self, 1))
    }

    /// Returns a [`StreamedParser`] by repeating the parser exactly `n` times.
    ///
    /// [`StreamedParser`]: streamed::StreamedParser
    #[inline]
    fn times(self, n: usize) -> Times<Self>
    where
        Self: Sized,
        I: Positioned,
    {
        assert_streamed_parser(Times::new(self, n))
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

    /// Returns a [`StreamedParser`] of sequenced the parser separated by `sep`.
    ///
    /// [`StreamedParser`]: streamed::StreamedParser
    #[inline]
    fn sep_by<P, R>(self, sep: P, range: R) -> SepBy<Self, P, R>
    where
        Self: Sized,
        P: Parser<I>,
        R: RangeBounds<usize>,
        I: Input,
    {
        assert_streamed_parser(SepBy::new(self, sep, range))
    }

    /// Returns a [`StreamedParser`] of sequenced the parser separated by `sep` (trailing separater is
    /// allowed).
    ///
    /// [`StreamedParser`]: streamed::StreamedParser
    #[inline]
    fn sep_by_end<P, R>(self, sep: P, range: R) -> SepByEnd<Self, P, R>
    where
        Self: Sized,
        P: Parser<I>,
        R: RangeBounds<usize>,
        I: Input,
    {
        assert_streamed_parser(SepByEnd::new(self, sep, range))
    }

    /// Returns a [`StreamedParser`] by repeating the parser until the parser `end` succeeds.
    ///
    /// [`StreamedParser`]: streamed::StreamedParser
    #[inline]
    fn until<P>(self, end: P) -> Until<Self, P>
    where
        Self: Sized,
        P: Parser<I>,
        I: Input,
    {
        assert_streamed_parser(Until::new(self, end))
    }

    /// Check an output value with the function.
    #[inline]
    fn satisfy<F, O>(self, f: F) -> Satisfy<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Output) -> bool,
    {
        assert_parser(Satisfy::new(self, f))
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

    /// Converting an output value into another type with a failable function.
    #[inline]
    fn try_map<F, O, E>(self, f: F) -> TryMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> Result<O, E>,
        E: Into<Expects<I::Ok>>,
    {
        assert_parser(TryMap::new(self, f))
    }

    /// Parses with `self`, passes output to the function `f` and parses with a returned parser.
    #[inline]
    fn then<F, Q>(self, f: F) -> Then<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> Q,
        Q: Parser<I>,
    {
        assert_parser(Then::new(self, f))
    }

    /// Parses with `self`, passes output to the failable function `f` and parses with a returned parser.
    #[inline]
    fn try_then<F, Q, E>(self, f: F) -> TryThen<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> Result<Q, E>,
        Q: Parser<I>,
        E: Into<Expects<I::Ok>>,
    {
        assert_parser(TryThen::new(self, f))
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

    /// Overriding the error position with the span of tokens parsed by the parser.
    #[inline]
    fn spanned(self) -> Spanned<Self>
    where
        Self: Sized,
    {
        assert_parser(Spanned::new(self))
    }

    /// Modifying the flag [`fatal`] for the error.
    ///
    /// If the flag is `true`, combinators like [`opt`], [`or`] or [`repeat`] will give special
    /// respects for errors originated by the parser so that the errors will not be ignored.
    /// Otherwise, the error can be ignored in specific situations.
    ///
    /// For example, [`opt`] rewinds input and returns [`None`], or [`or`] tries other choices when
    /// the parser returns an error.
    ///
    /// [`fatal`]: crate::error::ParseError::Parser::fatal
    /// [`opt`]: Self::opt
    /// [`or`]: Self::or
    /// [`repeat`]: Self::repeat
    #[inline]
    fn fatal(self, fatal: bool) -> Fatal<Self>
    where
        Self: Sized,
    {
        assert_parser(Fatal::new(self, fatal))
    }

    /// A conventional function to override parsing errors.
    ///
    /// This function override expected values (by the passed value), the position (by the position of
    /// parsed tokens), and [`fatal`] flag (by `false`) at once.
    ///
    /// [`map_err`]: Self::map_err
    /// [`fatal`]: crate::error::ParseError::Parser::fatal
    #[inline]
    fn expect<E: Into<Expects<I::Ok>>>(self, expected: E) -> Expect<Self, Expects<I::Ok>>
    where
        Self: Sized,
        I::Ok: Clone,
    {
        assert_parser(Expect::new(self, expected))
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
    ) -> Poll<ParseResult<Self::Output, I>> {
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
    ) -> Poll<ParseResult<Self::Output, I>> {
        (**self).poll_parse(input, cx, state, tracker)
    }
}

#[inline]
fn assert_parser<P: Parser<I>, I: Positioned + ?Sized>(parser: P) -> P {
    parser
}
