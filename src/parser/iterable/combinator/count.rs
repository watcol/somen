use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::iterable::IterableParser;
use crate::parser::utils::merge_errors;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`count`].
///
/// [`count`]: crate::parser::iterable::IterableParserExt::count
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Count<P> {
    inner: P,
}

impl<P> Count<P> {
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
    pub struct CountState<I, P: IterableParser> {
        inner: P::State,
        count: usize,
        error: Option<Error<I::Locator>>,
    }
}

impl<P, I> Parser<I> for Count<P>
where
    P: IterableParser<I>,
    I: Positioned + ?Sized,
{
    type Output = usize;
    type State = CountState<I, P>;

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
                Status::Success(Some(_), err) => {
                    state.count += 1;
                    merge_errors(&mut state.error, err);
                }
                Status::Success(None, err) => {
                    merge_errors(&mut state.error, err);
                    break Status::Success(state.count, state.error());
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    break Status::Failure(state.error().unwrap(), false);
                }
                Status::Failure(err, true) => {
                    break Status::Failure(err, true);
                }
            }
        }))
    }
}
