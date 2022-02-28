use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Expect, Expects, ParseError, PolledResult, Tracker};
use crate::parser::Parser;
use crate::stream::Input;

use super::utils::SpanState;

/// A parser for method [`peek`].
///
/// [`peek`]: super::ParserExt::peek
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Peek<P> {
    inner: P,
}

impl<P> Peek<P> {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeekState<C, M> {
    inner: C,
    queued_marker: Option<M>,
}

impl<C: Default, M> Default for PeekState<C, M> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: C::default(),
            queued_marker: None,
        }
    }
}

impl<P, I> Parser<I> for Peek<P>
where
    P: Parser<I>,
    I: Input + ?Sized,
{
    type Output = P::Output;
    type State = PeekState<P::State, I::Marker>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        if state.queued_marker.is_none() {
            state.queued_marker = Some(input.as_mut().mark()?);
        }

        Poll::Ready(
            match ready!(self
                .inner
                .poll_parse(input.as_mut(), cx, &mut state.inner, tracker))
            {
                Ok((i, _)) => {
                    input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    Ok((i, false))
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    Err(err)
                }
            },
        )
    }
}

/// A parser for method [`fail`].
///
/// [`fail`]: super::ParserExt::fail
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fail<P> {
    inner: P,
}

impl<P> Fail<P> {
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

impl<P, I> Parser<I> for Fail<P>
where
    P: Parser<I>,
    I: Input + ?Sized,
{
    type Output = ();
    type State = SpanState<PeekState<P::State, I::Marker>, I::Locator>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        state.set_start(|| input.position());
        if state.inner.queued_marker.is_none() {
            state.inner.queued_marker = Some(input.as_mut().mark()?);
        }

        Poll::Ready(
            match ready!(self
                .inner
                .poll_parse(input.as_mut(), cx, &mut state.inner.inner, tracker))
            {
                Ok(_) => {
                    input
                        .as_mut()
                        .drop_marker(mem::take(&mut state.inner.queued_marker).unwrap())?;
                    Err(ParseError::Parser {
                        expects: Expects::new(Expect::Static("<failure>")),
                        position: state.take_start()..input.position(),
                        fatal: false,
                    })
                }
                Err(ParseError::Parser { fatal: true, .. }) => {
                    input.rewind(mem::take(&mut state.inner.queued_marker).unwrap())?;
                    Ok(((), false))
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.inner.queued_marker).unwrap())?;
                    Err(err)
                }
            },
        )
    }
}
