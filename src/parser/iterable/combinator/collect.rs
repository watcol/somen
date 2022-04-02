use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::iterable::IterableParser;
use crate::parser::utils::merge_errors;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`collect`].
///
/// [`collect`]: crate::parser::iterable::IterableParserExt::collect
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collect<P, E> {
    inner: P,
    _phantom: PhantomData<E>,
}

impl<P, E> Collect<P, E> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct CollectState<I, P: IterableParser; E: Default> {
        inner: P::State,
        collection: E,
        reserved: bool,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, E, I> Parser<I> for Collect<P, E>
where
    P: IterableParser<I>,
    E: Extend<P::Item> + Default,
    I: Positioned + ?Sized,
{
    type Output = E;
    type State = CollectState<I, P, E>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        #[cfg(feature = "nightly")]
        if !state.reserved {
            state.collection.extend_reserve(self.inner.size_hint().0)
        }

        Poll::Ready(Ok(loop {
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
            {
                Status::Success(Some(val), err) => {
                    #[cfg(feature = "nightly")]
                    {
                        state.collection.extend_one(val);
                    }
                    #[cfg(not(feature = "nightly"))]
                    {
                        state.collection.extend(Some(val));
                    }
                    merge_errors(&mut state.error, err);
                }
                Status::Success(None, err) => {
                    merge_errors(&mut state.error, err);
                    break Status::Success(state.collection(), state.error());
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
