use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::merge_errors;
use crate::stream::Positioned;

/// A parser for method [`filter`].
///
/// [`filter`]: crate::parser::streamed::StreamedParserExt::filter
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter<P, F> {
    inner: P,
    f: F,
}

impl<P, F> Filter<P, F> {
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
    pub struct FilterState<I, P: StreamedParser> {
        inner: P::State,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, F, I> StreamedParser<I> for Filter<P, F>
where
    P: StreamedParser<I>,
    F: FnMut(&P::Item) -> bool,
    I: Positioned + ?Sized,
{
    type Item = P::Item;
    type State = FilterState<I, P>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        Poll::Ready(Ok(loop {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
            {
                (Status::Success(Some(val), err), pos) if (self.f)(&val) => {
                    merge_errors(&mut state.error, err);
                    state.set_start(|| pos.start);
                    break (
                        Status::Success(Some(val), state.error()),
                        state.start()..pos.end,
                    );
                }
                (Status::Success(Some(_), err), pos) => {
                    merge_errors(&mut state.error, err);
                    state.set_start(|| pos.start);
                }
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err);
                    state.set_start(|| pos.start);
                    break (Status::Success(None, state.error()), state.start()..pos.end);
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err));
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
