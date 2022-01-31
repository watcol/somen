use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;
use crate::stream::Input;

/// A parser for method [`opt`].
///
/// [`opt`]: super::ParserExt::opt
#[derive(Debug)]
pub struct Opt<P> {
    inner: P,
}

impl<P> Opt<P> {
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

#[derive(Debug)]
pub struct OptState<C, M> {
    inner: C,
    queued_marker: Option<M>,
}

impl<C: Default, M> Default for OptState<C, M> {
    fn default() -> Self {
        Self {
            inner: C::default(),
            queued_marker: None,
        }
    }
}

impl<P, I> Parser<I> for Opt<P>
where
    P: Parser<I>,
    I: Input + ?Sized,
{
    type Output = Option<P::Output>;
    type State = OptState<P::State, I::Marker>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self::Output, I>> {
        if state.queued_marker.is_none() {
            state.queued_marker = Some(input.as_mut().mark()?);
        }

        Poll::Ready(
            match ready!(self.inner.poll_parse(input.as_mut(), cx, &mut state.inner)) {
                Ok(i) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    Ok(Some(i))
                }
                Err(ParseError::Parser(_, _)) => {
                    input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    Ok(None)
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    Err(err)
                }
            },
        )
    }
}
