use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::merge_errors;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`collect`].
///
/// [`collect`]: crate::parser::streamed::StreamedParserExt::collect
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collect<P, E> {
    inner: P,
    _phantom: PhantomData<E>,
}

impl<P, E> Collect<P, E> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct CollectState<I, P: StreamedParser; E: Default> {
        inner: P::State,
        collection: E,
        reserved: bool,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, E, I> Parser<I> for Collect<P, E>
where
    P: StreamedParser<I>,
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
                (Status::Success(Some(val), err), pos) => {
                    #[cfg(feature = "nightly")]
                    {
                        state.collection.extend_one(val);
                    }
                    #[cfg(not(feature = "nightly"))]
                    {
                        state.collection.extend(Some(val));
                    }
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                }
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                    break (
                        Status::Success(state.collection(), state.error()),
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
