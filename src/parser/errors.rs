use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{Expects, ParseError, ParseResult};
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
    ) -> Poll<ParseResult<Self::Output, I>> {
        self.inner
            .poll_parse(input, cx, state)
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

/// A parser for method [`expect`].
///
/// [`expect`]: super::ParserExt::expect
#[derive(Debug)]
pub struct Expect<P> {
    inner: P,
    message: &'static str,
}

impl<P> Expect<P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, message: &'static str) -> Self {
        Self { inner, message }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, I> Parser<I> for Expect<P>
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
    ) -> Poll<ParseResult<Self::Output, I>> {
        self.inner
            .poll_parse(input, cx, state)
            .map_err(|err| match err {
                ParseError::Parser {
                    position, fatal, ..
                } => ParseError::Parser {
                    expects: Expects::new(crate::error::Expect::Static(self.message)),
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
    ) -> Poll<ParseResult<Self::Output, I>> {
        let start = input.position();
        self.inner
            .poll_parse(input.as_mut(), cx, state)
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
