use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::merge_errors;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`count`].
///
/// [`count`]: crate::parser::streamed::StreamedParserExt::count
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Count<P> {
    inner: P,
}

impl<P> Count<P> {
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
    pub struct CountState<I, P: StreamedParser> {
        inner: P::State,
        count: usize,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, I> Parser<I> for Count<P>
where
    P: StreamedParser<I>,
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
                (Status::Success(Some(_), err), pos) => {
                    state.count += 1;
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                }
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);

                    let start = if state.start.is_some() {
                        state.start()
                    } else {
                        pos.start
                    };

                    break (Status::Success(state.count, state.error()), start..pos.end);
                }
                (Status::Failure(err, exclusive), pos) => {
                    if exclusive {
                        state.error = Some(err);
                    } else {
                        merge_errors(&mut state.error, Some(err), &pos)
                    }

                    let start = if state.start.is_some() {
                        state.start()
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
