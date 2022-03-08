use core::mem;
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
    pub struct FailState<I: Input, P: Parser> {
        inner: P::State,
        #[state(option)]
        marker: I::Marker,
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
        if state.marker.is_none() {
            state.marker = Some(input.as_mut().mark()?);
        }

        Poll::Ready(Ok(
            match ready!(self.inner.poll_parse(input.as_mut(), cx, &mut state.inner,))? {
                (Status::Success(_, _), pos) => {
                    input.drop_marker(mem::take(&mut state.marker).unwrap())?;
                    (
                        Status::Failure(
                            Error {
                                expects: Expects::from("<failure>"),
                                position: pos.clone(),
                            },
                            false,
                        ),
                        pos,
                    )
                }
                (Status::Failure(_, false), pos) => {
                    input.rewind(mem::take(&mut state.marker).unwrap())?;
                    (Status::Success((), None), pos.start.clone()..pos.start)
                }
                (Status::Failure(err, true), pos) => {
                    input.drop_marker(mem::take(&mut state.marker).unwrap())?;
                    (Status::Failure(err, true), pos)
                }
            },
        ))
    }
}
