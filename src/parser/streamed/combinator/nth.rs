use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::merge_errors;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`nth`].
///
/// [`nth`]: crate::parser::streamed::StreamedParserExt::nth
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nth<P> {
    inner: P,
    n: usize,
}

impl<P> Nth<P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, n: usize) -> Self {
        Self { inner, n }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct NthState<I, P: StreamedParser> {
        inner: P::State,
        count: usize,
        output: Option<P::Item>,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, I> Parser<I> for Nth<P>
where
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Output = Option<P::Item>;
    type State = NthState<I, P>;

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
                    if state.count == self.n {
                        state.output = Some(val);
                    }
                    state.count += 1;
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                }
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                    break (
                        Status::Success(state.output(), state.error()),
                        state.start()..pos.end,
                    );
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err), &pos);
                    state.set_start(|| pos.start);
                    break (
                        Status::Failure(state.error().unwrap(), false),
                        state.start()..pos.end,
                    );
                }
                (Status::Failure(err, true), pos) => {
                    state.set_start(|| pos.start);
                    break (Status::Failure(err, true), state.start()..pos.end);
                }
            }
        }))
    }
}