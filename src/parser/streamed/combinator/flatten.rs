use core::ops::Range;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::merge_errors;
use crate::stream::Positioned;

/// A parser for method [`flatten`].
///
/// [`flatten`]: crate::parser::streamed::StreamedParserExt::flatten
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flatten<P> {
    inner: P,
}

impl<P> Flatten<P> {
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
    pub struct FlattenState<I, P: StreamedParser; T> {
        inner: P::State,
        iter: Option<T>,
        #[opt]
        pos: Range<I::Locator>,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, I> StreamedParser<I> for Flatten<P>
where
    P: StreamedParser<I>,
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
                    break (Status::Success(Some(val), state.error()), state.pos());
                }
            }

            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
            {
                (Status::Success(Some(iter), err), pos) => {
                    state.iter = Some(iter.into_iter());
                    merge_errors(&mut state.error, err, &pos);
                    state.pos = Some(if state.pos.is_some() {
                        state.pos().start..pos.end
                    } else {
                        pos
                    });
                }
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);

                    let start = if state.pos.is_some() {
                        state.pos().start
                    } else {
                        pos.start
                    };

                    break (Status::Success(None, state.error()), start..pos.end);
                }
                (Status::Failure(err, exclusive), pos) => {
                    if exclusive {
                        state.error = Some(err);
                    } else {
                        merge_errors(&mut state.error, Some(err), &pos);
                    }

                    let start = if state.pos.is_some() {
                        state.pos().start
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
