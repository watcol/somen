use core::pin::Pin;
use core::task::Context;

use crate::error::{PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::stream::Positioned;

/// A parser for method [`enumerate`].
///
/// [`enumerate`]: crate::parser::streamed::StreamedParserExt::enumerate
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Enumerate<P> {
    inner: P,
}

impl<P> Enumerate<P> {
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
    pub struct EnumerateState<I, P: StreamedParser> {
        inner: P::State,
        count: usize,
    }
}

impl<P, I> StreamedParser<I> for Enumerate<P>
where
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Item = (usize, P::Item);
    type State = EnumerateState<I, P>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        self.inner
            .poll_parse_next(input.as_mut(), cx, &mut state.inner)
            .map_ok(|(status, pos)| {
                (
                    match status {
                        Status::Success(Some(val), err) => {
                            let i = state.count;
                            state.count += 1;
                            Status::Success(Some((i, val)), err)
                        }
                        Status::Success(None, err) => Status::Success(None, err),
                        Status::Failure(err, exclusive) => Status::Failure(err, exclusive),
                    },
                    pos,
                )
            })
    }
}
