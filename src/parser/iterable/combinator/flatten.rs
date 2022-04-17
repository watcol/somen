use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::iterable::IterableParser;
use crate::parser::utils::merge_errors;
use crate::stream::Positioned;

/// A parser for method [`flatten`].
///
/// [`flatten`]: crate::parser::iterable::IterableParserExt::flatten
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flatten<P> {
    inner: P,
}

impl<P> Flatten<P> {
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
    pub struct FlattenState<I, P: IterableParser; T> {
        inner: P::State,
        iter: Option<T>,
        error: Option<Error<I::Locator>>,
    }
}

impl<P, I> IterableParser<I> for Flatten<P>
where
    P: IterableParser<I>,
    P::Item: IntoIterator,
    I: Positioned + ?Sized,
{
    type Item = <P::Item as IntoIterator>::Item;
    type State = FlattenState<I, P, <P::Item as IntoIterator>::IntoIter>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        Poll::Ready(Ok(loop {
            if let Some(iter) = &mut state.iter {
                if let Some(val) = iter.next() {
                    break Status::Success(Some(val), state.error());
                }
            }

            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
            {
                Status::Success(Some(iter), err) => {
                    state.iter = Some(iter.into_iter());
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
