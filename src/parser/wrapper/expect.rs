use core::pin::Pin;
use core::task::Context;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`expect`].
///
/// [`expect`]: crate::parser::ParserExt::expect
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expect<P> {
    inner: P,
    expects: Expects,
}

impl<P> Expect<P> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, expects: Expects) -> Self {
        Self { inner, expects }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct ExpectState<I, P: Parser> {
        inner: P::State,
        #[opt(set = set_start)]
        start: I::Locator,
    }
}

impl<P, I> Parser<I> for Expect<P>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = ExpectState<I, P>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        state.set_start(|| input.position());
        self.inner
            .poll_parse(input.as_mut(), cx, &mut state.inner)
            .map_ok(|status| match status {
                Status::Failure(err, false) if err.rewindable(&state.start()) => Status::Failure(
                    Error {
                        expects: self.expects.clone(),
                        position: err.position,
                    },
                    false,
                ),
                res => res,
            })
    }
}
