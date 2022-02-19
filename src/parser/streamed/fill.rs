use core::mem::{self, MaybeUninit};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Expect, Expects, ParseError, ParseResult, Tracker};
use crate::parser::utils::SpanState;
use crate::parser::Parser;
use crate::stream::Positioned;

use super::StreamedParser;

/// A parser for method [`fill`].
///
/// [`fill`]: super::StreamedParserExt::fill
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fill<P, const N: usize> {
    inner: P,
}

impl<P, const N: usize> Fill<P, N> {
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

#[derive(Debug)]
pub struct FillState<C, T, const N: usize> {
    inner: C,
    count: usize,
    buf: [MaybeUninit<T>; N],
}

impl<C: Default, T, const N: usize> Default for FillState<C, T, N> {
    fn default() -> Self {
        Self {
            inner: C::default(),
            count: 0,
            buf: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
}

impl<P, I, const N: usize> Parser<I> for Fill<P, N>
where
    P: StreamedParser<I>,
    I: Positioned + ?Sized,
{
    type Output = [P::Item; N];
    type State = SpanState<FillState<P::State, P::Item, N>, I::Locator>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
        tracker: &mut Tracker<I::Ok>,
    ) -> Poll<ParseResult<Self::Output, I>> {
        loop {
            state.set_start(|| input.position());
            match ready!(self.inner.poll_parse_next(
                input.as_mut(),
                cx,
                &mut state.inner.inner,
                tracker
            )?) {
                Some(x) if state.inner.count < N => {
                    state.inner.buf[state.inner.count].write(x);
                    state.inner.count += 1;
                    state.start = None;
                }
                None if state.inner.count == N => {
                    let mut buf = mem::replace(&mut state.inner.buf, unsafe {
                        MaybeUninit::uninit().assume_init()
                    });
                    let ptr = &mut buf as *mut _ as *mut [P::Item; N];
                    let res = unsafe { ptr.read() };
                    core::mem::forget(buf);
                    break Poll::Ready(Ok(res));
                }
                Some(_) => {
                    break Poll::Ready(Err(ParseError::Parser {
                        expects: Expects::new(Expect::Static("<end of stream>")),
                        position: state.take_start()..input.position(),
                        fatal: true,
                    }))
                }
                None => {
                    break Poll::Ready(Err(ParseError::Parser {
                        expects: Expects::new(Expect::Static("<more elements>")),
                        position: state.take_start()..input.position(),
                        fatal: true,
                    }))
                }
            }
        }
    }
}
