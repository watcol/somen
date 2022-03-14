use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::merge_errors;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`discard`].
///
/// [`discard`]: crate::parser::streamed::StreamedParserExt::discard
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

crate::parser_state! {
    pub struct DiscardState<I, P: StreamedParser> {
        inner: P::State,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, I> Parser<I> for Discard<P>
where
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Output = ();
    type State = DiscardState<I, P>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        Poll::Ready(Ok(loop {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
            {
                (Status::Success(Some(_), err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                }
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);

                    let start = if state.start.is_some() {
                        state.start()
                    } else {
                        pos.start
                    };

                    break (Status::Success((), state.error()), start..pos.end);
                }
                (Status::Failure(err, exclusive), pos) => {
                    if exclusive {
                        state.error = Some(err);
                    } else {
                        merge_errors(&mut state.error, Some(err), &pos)
                    }

                    let start = if state.start.is_some() {
                        state.start()
                    } else {
                        pos.start
                    };

                    break (
                        Status::Failure(state.error().unwrap(), exclusive),
                        start..pos.end,
                    );
                }
            }
        }))
    }
}
