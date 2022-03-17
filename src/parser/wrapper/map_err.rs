use core::pin::Pin;
use core::task::Context;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`map_err`].
///
/// [`map_err`]: crate::parser::ParserExt::map_err
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapErr<P, F> {
    inner: P,
    f: F,
}

impl<P, F> MapErr<P, F> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, F, E, I> Parser<I> for MapErr<P, F>
where
    P: Parser<I>,
    F: FnMut(Expects<I::Ok>) -> E,
    E: Into<Expects<I::Ok>>,
    I: Positioned + ?Sized,
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
            .map_ok(|status| match status {
                Status::Failure(Error { expects, position }, false) => Status::Failure(
                    Error {
                        expects: (self.f)(expects).into(),
                        position,
                    },
                    false,
                ),
                res => res,
            })
    }
}
