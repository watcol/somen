use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::Parser;
use crate::stream::Input;

/// A parser for method [`opt`].
///
/// [`opt`]: crate::parser::ParserExt::opt
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opt<P> {
    inner: P,
}

impl<P> Opt<P> {
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
    pub struct OptState<I: Input, P: Parser> {
        inner: P::State,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
        #[opt(set = set_start)]
        start: I::Locator,
    }
}

impl<P, I> Parser<I> for Opt<P>
where
    P: Parser<I>,
    I: Input + ?Sized,
{
    type Output = Option<P::Output>;
    type State = OptState<I, P>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        state.set_start(|| input.position());
        state.set_marker(|| input.as_mut().mark())?;

        Poll::Ready(Ok(
            match ready!(self
                .inner
                .poll_parse(input.as_mut(), cx, &mut state.inner)?)
            {
                Status::Success(val, err) => {
                    input.drop_marker(state.marker())?;
                    Status::Success(Some(val), err)
                }
                Status::Failure(err, false) if err.rewindable(&state.start()) => {
                    input.rewind(state.marker())?;
                    Status::Success(None, Some(err))
                }
                Status::Failure(err, exclusive) => Status::Failure(err, exclusive),
            },
        ))
    }
}

crate::parser_state! {
    pub struct OptStreamedState<I: Input, P: StreamedParser> {
        inner: P::State,
        #[opt(try_set = set_marker)]
        marker: I::Marker,
        succeeded: bool,
        #[opt(set = set_start)]
        start: I::Locator,
    }
}

impl<P, I> StreamedParser<I> for Opt<P>
where
    P: StreamedParser<I>,
    I: Input + ?Sized,
{
    type Item = P::Item;
    type State = OptStreamedState<I, P>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        if state.succeeded {
            return self.inner.poll_parse_next(input, cx, &mut state.inner);
        }

        state.set_start(|| input.position());
        state.set_marker(|| input.as_mut().mark())?;

        Poll::Ready(Ok(
            match ready!(self
                .inner
                .poll_parse_next(input.as_mut(), cx, &mut state.inner)?)
            {
                Status::Success(val, err) => {
                    input.drop_marker(state.marker())?;
                    Status::Success(val, err)
                }
                Status::Failure(err, false) if err.rewindable(&state.start()) => {
                    input.rewind(state.marker())?;
                    Status::Success(None, Some(err))
                }
                Status::Failure(err, exclusive) => Status::Failure(err, exclusive),
            },
        ))
    }
}
