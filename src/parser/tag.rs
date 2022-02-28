use core::marker::PhantomData;
use core::pin::Pin;
use core::str::Chars;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Expect, Expects, ParseError, PolledResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

use super::utils::SpanState;

/// A parser for function [`tag`].
///
/// [`tag`]: crate::parser::tag
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tag<I: ?Sized> {
    tag: &'static str,
    _phantom: PhantomData<I>,
}

impl<I: ?Sized> Tag<I> {
    /// Creating a new instance.
    #[inline]
    pub fn new(tag: &'static str) -> Self {
        Self {
            tag,
            _phantom: PhantomData,
        }
    }
}

impl<I> Parser<I> for Tag<I>
where
    I: Positioned<Ok = char> + ?Sized,
{
    type Output = &'static str;
    type State = SpanState<Option<Chars<'static>>, I::Locator>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> PolledResult<Self::Output, I> {
        state.set_start(|| input.position());
        let iter = state.inner.get_or_insert_with(|| self.tag.chars());
        loop {
            let val = match iter.next() {
                Some(i) => i,
                None => {
                    tracker.clear();
                    break Poll::Ready(Ok((self.tag, !self.tag.is_empty())));
                }
            };

            match ready!(input.as_mut().try_poll_next(cx)?) {
                Some(i) if i == val => continue,
                _ => {
                    break Poll::Ready(Err(ParseError::Parser {
                        expects: Expects::new(Expect::Static(self.tag)),
                        position: state.take_start()..input.position(),
                        fatal: false,
                    }))
                }
            }
        }
    }
}
