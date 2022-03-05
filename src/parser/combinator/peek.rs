use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Input;

/// A parser for method [`peek`].
///
/// [`peek`]: crate::parser::ParserExt::peek
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
    ) -> PolledResult<Self::Output, I> {
        if state.queued_marker.is_none() {
            state.queued_marker = Some(input.as_mut().mark()?);
        }

        Poll::Ready(Ok(
            match ready!(self.inner.poll_parse(input.as_mut(), cx, &mut state.inner))? {
                (Status::Success(val, err), pos) => {
                    input.rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    (Status::Success(val, err), pos.start.clone()..pos.start)
                }
                (Status::Failure(err, exclusive), pos) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    (Status::Failure(err, exclusive), pos)
                }
            },
        ))
    }
}
