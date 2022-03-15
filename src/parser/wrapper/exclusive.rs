use core::pin::Pin;
use core::task::Context;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`exclusive`].
///
/// [`exclusive`]: crate::parser::ParserExt::exclusive
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exclusive<P, E> {
    inner: P,
    expects: E,
}

impl<P, E> Exclusive<P, E> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, expects: E) -> Self {
        Self { inner, expects }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, I> Parser<I> for Exclusive<P, Expects<I::Ok>>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
    I::Ok: Clone,
{
    type Output = P::Output;
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
                        Status::Failure(_, false) => Status::Failure(
                            Error {
                                expects: self.expects.clone(),
                                position: pos.clone(),
                            },
                            true,
                        ),
                        res => res,
                    },
                    pos,
                )
            })
    }
}
