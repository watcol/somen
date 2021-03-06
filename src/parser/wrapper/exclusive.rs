use core::pin::Pin;
use core::task::Context;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`exclusive`].
///
/// [`exclusive`]: crate::parser::ParserExt::exclusive
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exclusive<P> {
    inner: P,
    expects: Expects,
}

impl<P> Exclusive<P> {
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
    pub struct ExclusiveState<I, P: Parser> {
        inner: P::State,
        #[opt(set = set_start)]
        start: I::Locator,
    }
}

impl<P, I> Parser<I> for Exclusive<P>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = ExclusiveState<I, P>;

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
                Status::Failure(_, false) => Status::Failure(
                    Error {
                        expects: self.expects.clone(),
                        position: state.start()..input.position(),
                    },
                    true,
                ),
                res => res,
            })
    }
}
