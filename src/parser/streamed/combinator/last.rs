use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::merge_errors;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`last`].
///
/// [`last`]: crate::parser::streamed::StreamedParserExt::last
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Last<P> {
    inner: P,
}

impl<P> Last<P> {
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
    pub struct LastState<I, P: StreamedParser> {
        inner: P::State,
        output: Option<P::Item>,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, I> Parser<I> for Last<P>
where
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Output = Option<P::Item>;
    type State = LastState<I, P>;

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
                (Status::Success(Some(val), err), pos) => {
                    state.output = Some(val);
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                }
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    break (
                        Status::Success(state.output(), state.error()),
                        state.start()..pos.end,
                    );
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err), &pos);
                    break (Status::Failure(state.error().unwrap(), false), pos);
                }
                (Status::Failure(err, true), pos) => break (Status::Failure(err, true), pos),
            }
        }))
    }
}
