use core::marker::PhantomData;
use core::pin::Pin;
use core::str::Chars;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, Expects, PolledResult, Status};
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for function [`tag`].
///
/// [`tag`]: crate::parser::tag
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tag<I: ?Sized> {
    tag: &'static str,
    _phantom: PhantomData<I>,
}

impl<I: ?Sized> Tag<I> {
    /// Creates a new instance.
    #[inline]
    pub fn new(tag: &'static str) -> Self {
        Self {
            tag,
            _phantom: PhantomData,
        }
    }
}

crate::parser_state! {
    pub struct TagState<I> {
        #[opt]
        iter: Chars<'static>,
        #[opt(set = set_start)]
        start: I::Locator,
        #[opt]
        next: I::Locator,
    }
}

impl<I> Parser<I> for Tag<I>
where
    I: Positioned + ?Sized,
    I::Ok: PartialEq<char>,
{
    type Output = &'static str;
    type State = TagState<I>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        state.set_start(|| input.position());
        let iter = state.iter.get_or_insert_with(|| self.tag.chars());
        Poll::Ready(Ok(loop {
            let val = match iter.next() {
                Some(i) => i,
                None => break Status::Success(self.tag, None),
            };

            let parsed = ready!(input.as_mut().try_poll_next(cx)?);
            state.next.get_or_insert_with(|| input.position());

            match parsed {
                Some(i) if i == val => continue,
                _ => {
                    break Status::Failure(
                        Error {
                            expects: Expects::from(self.tag),
                            position: state.start()..state.next(),
                        },
                        false,
                    )
                }
            }
        }))
    }
}
