use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{PolledResult, Status};
use crate::parser::Parser;
use crate::prelude::{Positioned, StreamedParser};

/// A streamed parser generated from method [`times`].
///
/// [`times`]: crate::parser::ParserExt::times
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Times<P> {
    inner: P,
    count: usize,
}

impl<P> Times<P> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, count: usize) -> Self {
        Self { inner, count }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct TimesState<I, P: Parser> {
        inner: P::State,
        count: usize,
    }
}

impl<P, I> StreamedParser<I> for Times<P>
where
    P: Parser<I>,
    I: Positioned + ?Sized,
{
    type Item = P::Output;
    type State = TimesState<I, P>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        // Return `None` if the number of items already reached `end_bound`.
        if state.count >= self.count {
            let pos = input.position();
            return Poll::Ready(Ok((Status::Success(None, None), pos.clone()..pos)));
        }

        Poll::Ready(Ok(
            match ready!(self
                .inner
                .poll_parse(input.as_mut(), cx, &mut state.inner)?)
            {
                (Status::Success(val, err), pos) => {
                    state.count += 1;
                    (Status::Success(Some(val), err), pos)
                }
                (Status::Failure(err, exclusive), pos) => (Status::Failure(err, exclusive), pos),
            },
        ))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}
