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
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P) -> Self {
        Self { inner }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct PeekState<I: Input, P: Parser> {
        inner: P::State,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
    }
}

impl<P, I> Parser<I> for Peek<P>
where
    P: Parser<I>,
    I: Input + ?Sized,
{
    type Output = P::Output;
    type State = PeekState<I, P>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        state.set_marker(|| input.as_mut().mark())?;

        Poll::Ready(Ok(
            match ready!(self.inner.poll_parse(input.as_mut(), cx, &mut state.inner))? {
                Status::Success(val, err) => {
                    input.rewind(state.marker())?;
                    Status::Success(val, err)
                }
                Status::Failure(err, exclusive) => {
                    input.drop_marker(state.marker())?;
                    Status::Failure(err, exclusive)
                }
            },
        ))
    }
}
