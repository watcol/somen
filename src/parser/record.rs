use alloc::borrow::ToOwned;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::ParseResult;
use crate::parser::Parser;
use crate::stream::NoRewindInput;

/// A parser for method [`record`].
///
/// [`record`]: super::ParserExt::opt
#[derive(Debug)]
pub struct Record<P> {
    inner: P,
}

impl<P> Record<P> {
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

#[derive(Debug, Default)]
pub struct RecordState<C> {
    inner: C,
    started: bool,
}

impl<P, I> Parser<I> for Record<P>
where
    P: Parser<I>,
    I: NoRewindInput + ?Sized,
{
    type Output = <I::Borrowed as ToOwned>::Owned;
    type Error = P::Error;
    type State = RecordState<P::State>;

    fn poll_parse(
        &self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        if !state.started {
            input.as_mut().start();
            state.started = true;
        }

        self.inner
            .poll_parse(input.as_mut(), cx, &mut state.inner)
            .map_ok(|_| {
                state.started = false;
                input.end().into_owned()
            })
    }
}

/// A parser for method [`with_record`].
///
/// [`with_record`]: super::ParserExt::opt
#[derive(Debug)]
pub struct WithRecord<P> {
    inner: P,
}

impl<P> WithRecord<P> {
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

impl<P, I> Parser<I> for WithRecord<P>
where
    P: Parser<I>,
    I: NoRewindInput + ?Sized,
{
    type Output = (P::Output, <I::Borrowed as ToOwned>::Owned);
    type Error = P::Error;
    type State = RecordState<P::State>;

    fn poll_parse(
        &self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self, I>> {
        if !state.started {
            input.as_mut().start();
            state.started = true;
        }

        self.inner
            .poll_parse(input.as_mut(), cx, &mut state.inner)
            .map_ok(|o| {
                state.started = false;
                (o, input.end().into_owned())
            })
    }
}
