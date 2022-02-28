use core::pin::Pin;
use core::task::Context;

use super::utils::SpanState;
use crate::error::{Expect, Expects, ParseError, PolledResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`satisfy`].
///
/// [`satisfy`]: super::ParserExt::satisfy
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Satisfy<P, F> {
    inner: P,
    f: F,
}

impl<P, F> Satisfy<P, F> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, f: F) -> Self {
        Self { inner, f }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P, F, I> Parser<I> for Satisfy<P, F>
where
    P: Parser<I>,
    F: FnMut(&P::Output) -> bool,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = SpanState<P::State, I::Locator>;

    #[inline]
    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        state.set_start(|| input.position());
        self.inner
            .poll_parse(input.as_mut(), cx, &mut state.inner, tracker)
            .map(|res| {
                res.and_then(|(val, committed)| {
                    if (self.f)(&val) {
                        Ok((val, committed))
                    } else {
                        tracker.clear();
                        Err(ParseError::Parser {
                            expects: Expects::new(Expect::Static("<cond>")),
                            position: state.take_start()..input.position(),
                            fatal: true,
                        })
                    }
                })
            })
    }
}
