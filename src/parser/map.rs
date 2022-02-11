use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::{Expects, ParseError, ParseResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`map`].
///
/// [`map`]: super::ParserExt::map
#[derive(Debug)]
pub struct Map<P, F> {
    inner: P,
    f: F,
}

impl<P, F> Map<P, F> {
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

impl<P, F, I, O> Parser<I> for Map<P, F>
where
    P: Parser<I>,
    F: FnMut(P::Output) -> O,
    I: Positioned + ?Sized,
{
    type Output = O;
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
            .map_ok(&mut self.f)
    }
}

/// A parser for method [`try_map`].
///
/// [`try_map`]: super::ParserExt::try_map
#[derive(Debug)]
pub struct TryMap<P, F> {
    inner: P,
    f: F,
}

impl<P, F> TryMap<P, F> {
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

impl<P, F, I, O, E> Parser<I> for TryMap<P, F>
where
    P: Parser<I>,
    F: FnMut(P::Output) -> Result<O, E>,
    E: Into<Expects<I::Ok>>,
    I: Positioned + ?Sized,
{
    type Output = O;
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
            .map(|res| {
                res.and_then(|val| {
                    (self.f)(val).map_err(|err| ParseError::Parser {
                        expects: err.into(),
                        position: start..input.position(),
                        fatal: true,
                    })
                })
            })
    }
}
