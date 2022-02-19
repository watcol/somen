use core::pin::Pin;
use core::task::{Context, Poll};

use super::utils::SpanState;
use crate::error::{Expects, ParseError, ParseResult, Tracker};
use crate::parser::streamed::StreamedParser;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`map`].
///
/// [`map`]: super::ParserExt::map
#[derive(Clone, Debug, PartialEq, Eq)]
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

    #[inline]
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

impl<P, F, I, O> StreamedParser<I> for Map<P, F>
where
    P: StreamedParser<I>,
    F: FnMut(P::Item) -> O,
    I: Positioned + ?Sized,
{
    type Item = O;
    type State = P::State;

    #[inline]
    fn poll_parse_next(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        self.inner
            .poll_parse_next(input, cx, state, tracker)
            .map_ok(|res| res.map(&mut self.f))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A parser for method [`try_map`].
///
/// [`try_map`]: super::ParserExt::try_map
#[derive(Clone, Debug, PartialEq, Eq)]
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
    type State = SpanState<P::State, I::Locator>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        state.set_start(|| input.position());
        self.inner
            .poll_parse(input.as_mut(), cx, &mut state.inner, tracker)
            .map(|res| {
                res.and_then(|val| {
                    (self.f)(val).map_err(|err| {
                        tracker.clear();
                        ParseError::Parser {
                            expects: err.into(),
                            position: state.take_start()..input.position(),
                            fatal: true,
                        }
                    })
                })
            })
    }
}

impl<P, F, I, O, E> StreamedParser<I> for TryMap<P, F>
where
    P: StreamedParser<I>,
    F: FnMut(P::Item) -> Result<O, E>,
    E: Into<Expects<I::Ok>>,
    I: Positioned + ?Sized,
{
    type Item = O;
    type State = SpanState<P::State, I::Locator>;

    #[inline]
    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Option<Self::Item>, I>> {
        state.set_start(|| input.position());
        self.inner
            .poll_parse_next(input.as_mut(), cx, &mut state.inner, tracker)
            .map(|res| {
                res.and_then(|item| match item {
                    Some(val) => (self.f)(val).map(Some).map_err(|err| {
                        tracker.clear();
                        ParseError::Parser {
                            expects: err.into(),
                            position: state.take_start()..input.position(),
                            fatal: true,
                        }
                    }),
                    None => Ok(None),
                })
            })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
