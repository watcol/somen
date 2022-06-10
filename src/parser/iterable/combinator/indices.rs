use core::mem::MaybeUninit;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::iterable::IterableParser;
use crate::parser::utils::merge_errors;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`indices`].
///
/// [`indices`]: crate::parser::iterable::IterableParserExt::indices
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Indices<P, const N: usize> {
    inner: P,
    ns: [usize; N],
}

impl<P, const N: usize> Indices<P, N> {
    /// Creates a new instance.
    #[inline]
    pub fn new(inner: P, ns: [usize; N]) -> Self {
        Self { inner, ns }
    }

    /// Creates a new instance for method [`fill`].
    ///
    /// [`fill`]: crate::parser::iterable::IterableParserExt::fill
    pub fn new_fill(inner: P, start: usize) -> Self {
        let mut ns = UninitBuffer::default();

        for i in start..start + N {
            ns.push(i);
        }

        Self {
            inner,
            ns: ns.take(),
        }
    }

    /// Extracts the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct IndexesState<I, P: IterableParser | const N: usize> {
        inner: P::State,
        count: usize,
        buf: UninitBuffer<P::Item, N>,
        end: bool,
        error: Option<Error<I::Locator>>,
    }
}

impl<P, I, const N: usize> Parser<I> for Indices<P, N>
where
    P: IterableParser<I>,
    I: Positioned + ?Sized,
{
    type Output = Option<[P::Item; N]>;
    type State = IndexesState<I, P, N>;

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
                Status::Success(Some(val), err) if !state.buf.is_filled() => {
                    let index = self.ns[state.buf.index()];
                    if state.count == index {
                        state.buf.push(val);
                        if !state.buf.is_filled() && self.ns[state.buf.index()] <= index {
                            panic!("ns must be ascending ordered.");
                        }
                    }
                    state.count += 1;
                    merge_errors(&mut state.error, err);
                }
                Status::Success(Some(_), err) => {
                    merge_errors(&mut state.error, err);
                }
                Status::Success(None, err) if state.buf.is_filled() => {
                    merge_errors(&mut state.error, err);
                    break Status::Success(Some(state.buf.take()), state.error());
                }
                Status::Success(None, err) => {
                    merge_errors(&mut state.error, err);
                    break Status::Success(None, state.error());
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    break Status::Failure(state.error().unwrap(), false);
                }
                Status::Failure(err, true) => break Status::Failure(err, true),
            }
        }))
    }
}

pub struct UninitBuffer<T, const N: usize> {
    index: usize,
    buf: [MaybeUninit<T>; N],
}

impl<T, const N: usize> Default for UninitBuffer<T, N> {
    fn default() -> Self {
        Self {
            index: 0,
            buf: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
}

impl<T, const N: usize> UninitBuffer<T, N> {
    #[inline]
    fn index(&self) -> usize {
        self.index
    }

    #[inline]
    fn is_filled(&self) -> bool {
        self.index == N
    }

    fn push(&mut self, val: T) {
        if self.is_filled() {
            panic!("no more values can be inserted");
        }

        self.buf[self.index].write(val);
        self.index += 1;
    }

    fn take(&mut self) -> [T; N] {
        if !self.is_filled() {
            panic!("The buffer must be filled.");
        }
        self.index = 0;
        let mut buf = core::mem::replace(&mut self.buf, unsafe {
            MaybeUninit::uninit().assume_init()
        });
        let ptr = &mut buf as *mut _ as *mut [T; N];
        let res = unsafe { ptr.read() };
        core::mem::forget(buf);
        res
    }
}
