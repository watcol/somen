use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{Expects, ParseError, ParseResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`map_err`].
///
/// [`map_err`]: super::ParserExt::map_err
#[derive(Debug)]
pub struct MapErr<P, F> {
    inner: P,
    f: F,
}

impl<P, F> MapErr<P, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, F, E, I> Parser<I> for MapErr<P, F>
where
    P: Parser<I>,
    F: FnMut(Expects<I::Ok>) -> E,
    E: Into<Expects<I::Ok>>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = P::State;

    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        self.inner
            .poll_parse(input, cx, state, tracker)
            .map_err(|err| match err {
                ParseError::Parser {
                    expects,
                    position,
                    fatal,
                } => ParseError::Parser {
                    expects: (self.f)(expects).into(),
                    position,
                    fatal,
                },
                ParseError::Stream(e) => ParseError::Stream(e),
            })
    }
}

/// A parser for method [`spanned`].
///
/// [`spanned`]: super::ParserExt::spanned
#[derive(Debug)]
pub struct Spanned<P> {
    inner: P,
}

impl<P> Spanned<P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P) -> Self {
        Self { inner }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, I> Parser<I> for Spanned<P>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = P::State;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        let start = input.position();
        self.inner
            .poll_parse(input.as_mut(), cx, state, tracker)
            .map_err(|err| match err {
                ParseError::Parser { expects, fatal, .. } => ParseError::Parser {
                    expects,
                    position: start..input.position(),
                    fatal,
                },
                ParseError::Stream(e) => ParseError::Stream(e),
            })
    }
}

/// A parser for method [`fatal`].
///
/// [`fatal`]: super::ParserExt::fatal
#[derive(Debug)]
pub struct Fatal<P> {
    inner: P,
    fatal: bool,
}

impl<P> Fatal<P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, fatal: bool) -> Self {
        Self { inner, fatal }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, I> Parser<I> for Fatal<P>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = P::State;

    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        self.inner
            .poll_parse(input, cx, state, tracker)
            .map_err(|err| match err {
                ParseError::Parser {
                    expects, position, ..
                } => ParseError::Parser {
                    expects,
                    position,
                    fatal: self.fatal,
                },
                ParseError::Stream(e) => ParseError::Stream(e),
            })
    }
}

/// A parser for method [`expect`].
///
/// [`expect`]: super::ParserExt::expect
#[derive(Debug)]
pub struct Expect<P, E> {
    inner: P,
    expects: E,
}

impl<P, E> Expect<P, E> {
    /// Creating a new instance.
    #[inline]
    pub fn new<F: Into<E>>(inner: P, expects: F) -> Self {
        Self {
            inner,
            expects: expects.into(),
        }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, I> Parser<I> for Expect<P, Expects<I::Ok>>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
    I::Ok: Clone,
{
    type Output = P::Output;
    type State = P::State;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        let start = input.position();
        self.inner
            .poll_parse(input.as_mut(), cx, state, tracker)
            .map_err(|err| match err {
                ParseError::Parser { .. } => ParseError::Parser {
                    expects: self.expects.clone(),
                    position: start..input.position(),
                    fatal: false,
                },
                ParseError::Stream(e) => ParseError::Stream(e),
            })
    }
}
