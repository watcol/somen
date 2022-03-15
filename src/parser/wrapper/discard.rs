use core::pin::Pin;
use core::task::Context;

use crate::error::{PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`discard`].
///
/// [`discard`]: crate::parser::ParserExt::discard
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Discard<P> {
    inner: P,
}

impl<P> Discard<P> {
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

impl<P, I> Parser<I> for Discard<P>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = ();
    type State = P::State;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        self.inner
            .poll_parse(input.as_mut(), cx, state)
            .map_ok(|res| match res {
                (Status::Success(_, err), pos) => (Status::Success((), err), pos),
                (Status::Failure(err, exclusive), pos) => (Status::Failure(err, exclusive), pos),
            })
    }
}
