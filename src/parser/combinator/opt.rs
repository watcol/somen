use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Input;

/// A parser for method [`opt`].
///
/// [`opt`]: crate::parser::ParserExt::opt
#[derive(Clone, Debug, PartialEq, Eq)]
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

crate::parser_state! {
    pub struct OptState<I: Input, P: Parser> {
        inner: P::State,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
    }
}

impl<P, I> Parser<I> for Opt<P>
where
    P: Parser<I>,
    I: Input + ?Sized,
{
    type Output = Option<P::Output>;
    type State = OptState<I, P>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        state.set_marker(|| input.as_mut().mark())?;

        Poll::Ready(Ok(
            match ready!(self
                .inner
                .poll_parse(input.as_mut(), cx, &mut state.inner)?)
            {
                (Status::Success(val, err), pos) => {
                    input.drop_marker(state.marker())?;
                    (Status::Success(Some(val), err), pos)
                }
                (Status::Failure(err, false), pos) if err.rewindable(&pos.start) => {
                    input.rewind(state.marker())?;
                    (
                        Status::Success(None, Some(err)),
                        pos.start.clone()..pos.start,
                    )
                }
                (Status::Failure(err, exclusive), pos) => (Status::Failure(err, exclusive), pos),
            },
        ))
    }
}