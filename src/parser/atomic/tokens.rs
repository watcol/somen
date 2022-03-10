use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, ExpectKind, Expects, PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`tokens`].
///
/// [`tokens`]: crate::parser::tokens
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

crate::parser_state! {
    pub struct TokensState<I; T> {
        #[opt]
        iter: T,
        #[opt(set = set_start, get = get_start)]
        start: I::Locator,
        #[opt]
        next: I::Locator,
    }
}

impl<'a, I, T> Parser<I> for Tokens<'a, I, T>
where
    I: Positioned + ?Sized,
    I::Ok: PartialEq,
    T: IntoIterator<Item = &'a I::Ok> + Clone,
{
    type Output = T;
    type State = TokensState<I, T::IntoIter>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        state.set_start(|| input.position());
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
                                position: state.get_start().clone()..state.next(),
                            },
                            false,
                        )
                    }
                }
            },
            state.start()..input.position(),
        )))
    }
}
