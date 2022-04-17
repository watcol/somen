use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::iterable::IterableParser;
use crate::parser::utils::merge_errors;
use crate::stream::Positioned;

/// A parser for method [`filter`].
///
/// [`filter`]: crate::parser::iterable::IterableParserExt::filter
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter<P, F> {
    inner: P,
    f: F,
}

impl<P, F> Filter<P, F> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct FilterState<I, P: IterableParser> {
        inner: P::State,
        error: Option<Error<I::Locator>>,
    }
}

impl<P, F, I> IterableParser<I> for Filter<P, F>
where
    P: IterableParser<I>,
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
                Status::Success(Some(val), err) if (self.f)(&val) => {
                    merge_errors(&mut state.error, err);
                    break Status::Success(Some(val), state.error());
                }
                Status::Success(Some(_), err) => {
                    merge_errors(&mut state.error, err);
                }
                Status::Success(None, err) => {
                    merge_errors(&mut state.error, err);
                    break Status::Success(None, state.error());
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    break Status::Failure(state.error().unwrap(), false);
                }
                Status::Failure(err, true) => break Status::Failure(err, true),
            }
        }))
    }
}
