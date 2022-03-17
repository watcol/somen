use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Input;

/// A parser for method [`fail`].
///
/// [`fail`]: crate::parser::ParserExt::fail
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fail<P> {
    inner: P,
}

impl<P> Fail<P> {
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
    pub struct FailState<I: Input, P: Parser> {
        inner: P::State,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
        #[opt(set = set_start)]
        start: I::Locator,
    }
}

impl<P, I> Parser<I> for Fail<P>
where
    P: Parser<I>,
    I: Input + ?Sized,
{
    type Output = ();
    type State = FailState<I, P>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        state.set_marker(|| input.as_mut().mark())?;
        state.set_start(|| input.position());

        Poll::Ready(Ok(
            match ready!(self.inner.poll_parse(input.as_mut(), cx, &mut state.inner,))? {
                Status::Success(_, _) => {
                    input.as_mut().drop_marker(state.marker())?;
                    Status::Failure(
                        Error {
                            expects: Expects::from("<failure>"),
                            position: state.start()..input.position(),
                        },
                        false,
                    )
                }
                Status::Failure(_, false) => {
                    input.rewind(state.marker())?;
                    Status::Success((), None)
                }
                Status::Failure(err, true) => {
                    input.drop_marker(state.marker())?;
                    Status::Failure(err, true)
                }
            },
        ))
    }
}
