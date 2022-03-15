use core::ops::Range;
use core::pin::Pin;
use core::task::Context;

use crate::error::{PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`with_position`].
///
/// [`with_position`]: crate::parser::ParserExt::with_position
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WithPosition<P> {
    inner: P,
}

impl<P> WithPosition<P> {
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

impl<P, I> Parser<I> for WithPosition<P>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = (P::Output, Range<I::Locator>);
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
                        Status::Success(val, err) => Status::Success((val, pos.clone()), err),
                        Status::Failure(err, exclusive) => Status::Failure(err, exclusive),
                    },
                    pos,
                )
            })
    }
}
