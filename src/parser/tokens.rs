use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Expect, Expects, ParseError, ParseResult, Tracker};
use crate::parser::Parser;
use crate::stream::Positioned;

use super::utils::SpanState;

/// A parser for function [`token`].
///
/// [`token`]: crate::parser::token
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tokens<'a, I: ?Sized, T> {
    tokens: T,
    _phantom: PhantomData<&'a I>,
}

impl<'a, I: ?Sized, T> Tokens<'a, I, T> {
    /// Creating a new instance.
    #[inline]
    pub fn new(tokens: T) -> Self {
        Self {
            tokens,
            _phantom: PhantomData,
        }
    }
}

impl<'a, I, T> Parser<I> for Tokens<'a, I, T>
where
    I: Positioned + ?Sized,
    I::Ok: PartialEq,
    T: IntoIterator<Item = &'a I::Ok> + Clone,
{
    type Output = T;
    type State = SpanState<Option<T::IntoIter>, I::Locator>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        state.set_start(|| input.position());
        let iter = state
            .inner
            .get_or_insert_with(|| self.tokens.clone().into_iter());
        loop {
            let val = match iter.next() {
                Some(i) => i,
                None => {
                    tracker.clear();
                    break Poll::Ready(Ok(self.tokens.clone()));
                }
            };

            match ready!(input.as_mut().try_poll_next(cx)?) {
                Some(i) if i == *val => continue,
                _ => {
                    break Poll::Ready(Err(ParseError::Parser {
                        expects: Expects::new(Expect::Static("<tokens>")),
                        position: state.take_start()..input.position(),
                        fatal: false,
                    }))
                }
            }
        }
    }
}