use core::pin::Pin;
use core::task::Context;

use crate::error::{Error, PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`spanned`].
///
/// [`spanned`]: crate::parser::ParserExt::spanned
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spanned<P> {
    inner: P,
}

impl<P> Spanned<P> {
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
    pub struct SpannedState<I, P: Parser> {
        inner: P::State,
        #[opt(set = set_start)]
        start: I::Locator,
    }
}

impl<P, I> Parser<I> for Spanned<P>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = SpannedState<I, P>;

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
                Status::Failure(Error { expects, .. }, false) => Status::Failure(
                    Error {
                        expects,
                        position: state.start()..input.position(),
                    },
                    false,
                ),
                status => status,
            })
    }
}
