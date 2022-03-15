use core::pin::Pin;
use core::task::Context;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`map`].
///
/// [`map`]: crate::parser::ParserExt::map
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

impl<P, F, O, I> Parser<I> for Map<P, F>
where
    P: Parser<I>,
    F: FnMut(P::Output) -> O,
    I: Positioned + ?Sized,
{
    type Output = O;
    type State = P::State;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        self.inner
            .poll_parse(input.as_mut(), cx, state)
            .map_ok(|(status, pos)| {
                (
                    match status {
                        Status::Success(val, err) => Status::Success((self.f)(val), err),
                        Status::Failure(err, exclusive) => Status::Failure(err, exclusive),
                    },
                    pos,
                )
            })
    }
}

impl<P, F, T, I> StreamedParser<I> for Map<P, F>
where
    P: StreamedParser<I>,
    F: FnMut(P::Item) -> T,
    I: Positioned + ?Sized,
{
    type Item = T;
    type State = P::State;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        self.inner
            .poll_parse_next(input.as_mut(), cx, state)
            .map_ok(|(status, pos)| {
                (
                    match status {
                        Status::Success(val, err) => Status::Success(val.map(&mut self.f), err),
                        Status::Failure(err, exclusive) => Status::Failure(err, exclusive),
                    },
                    pos,
                )
            })
    }
}

/// A parser for method [`try_map`].
///
/// [`try_map`]: crate::parser::ParserExt::try_map
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

impl<P, F, O, E, I> Parser<I> for TryMap<P, F>
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
    ) -> PolledResult<Self::Output, I> {
        self.inner
            .poll_parse(input.as_mut(), cx, state)
            .map_ok(|(status, pos)| {
                (
                    match status {
                        Status::Success(val, err) => match (self.f)(val) {
                            Ok(res) => Status::Success(res, err),
                            Err(exp) => Status::Failure(
                                Error {
                                    expects: exp.into(),
                                    position: pos.clone(),
                                },
                                true,
                            ),
                        },
                        Status::Failure(err, exclusive) => Status::Failure(err, exclusive),
                    },
                    pos,
                )
            })
    }
}

impl<P, F, T, E, I> StreamedParser<I> for TryMap<P, F>
where
    P: StreamedParser<I>,
    F: FnMut(P::Item) -> Result<T, E>,
    E: Into<Expects<I::Ok>>,
    I: Positioned + ?Sized,
{
    type Item = T;
    type State = P::State;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        self.inner
            .poll_parse_next(input.as_mut(), cx, state)
            .map_ok(|(status, pos)| {
                (
                    match status {
                        Status::Success(Some(val), err) => match (self.f)(val) {
                            Ok(res) => Status::Success(Some(res), err),
                            Err(exp) => Status::Failure(
                                Error {
                                    expects: exp.into(),
                                    position: pos.clone(),
                                },
                                true,
                            ),
                        },
                        Status::Success(None, err) => Status::Success(None, err),
                        Status::Failure(err, exclusive) => Status::Failure(err, exclusive),
                    },
                    pos,
                )
            })
    }
}
