use core::pin::Pin;
use core::task::Context;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`satisfy`].
///
/// [`satisfy`]: crate::parser::ParserExt::satisfy
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Satisfy<P, F> {
    inner: P,
    f: F,
}

impl<P, F> Satisfy<P, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct SatisfyState<I, P: Parser> {
        inner: P::State,
        #[opt(set = set_start)]
        start: I::Locator,
    }
}

impl<P, F, I> Parser<I> for Satisfy<P, F>
where
    P: Parser<I>,
    F: FnMut(&P::Output) -> bool,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = SatisfyState<I, P>;

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
                Status::Success(val, err) if (self.f)(&val) => Status::Success(val, err),
                Status::Success(_, _) => Status::Failure(
                    Error {
                        expects: Expects::from("<condition>"),
                        position: state.start()..input.position(),
                    },
                    true,
                ),
                Status::Failure(err, exclusive) => Status::Failure(err, exclusive),
            })
    }
}
