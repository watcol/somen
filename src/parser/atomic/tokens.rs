use core::marker::PhantomData;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, ExpectKind, Expects, PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokensState<I, L> {
    iter: Option<I>,
    start: Option<L>,
    next: Option<L>,
}

impl<I, L> Default for TokensState<I, L> {
    #[inline]
    fn default() -> Self {
        Self {
            iter: None,
            start: None,
            next: None,
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
    type State = TokensState<T::IntoIter, I::Locator>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        state.start.get_or_insert_with(|| input.position());
        let iter = state
            .iter
            .get_or_insert_with(|| self.tokens.clone().into_iter());
        Poll::Ready(Ok((
            loop {
                let val = match iter.next() {
                    Some(i) => i,
                    None => break Status::Success(self.tokens.clone(), None),
                };

                let parsed = ready!(input.as_mut().try_poll_next(cx)?);
                state.next.get_or_insert_with(|| input.position());
                match parsed {
                    Some(i) if i == *val => continue,
                    _ => {
                        break Status::Failure(
                            Error {
                                expects: Expects::new(ExpectKind::Static("<tokens>")),
                                position: state.start.clone().unwrap()
                                    ..mem::take(&mut state.next).unwrap(),
                            },
                            false,
                        )
                    }
                }
            },
            mem::take(&mut state.start).unwrap()..input.position(),
        )))
    }
}
