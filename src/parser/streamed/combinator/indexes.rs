use core::mem::MaybeUninit;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::merge_errors;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`indexes`].
///
/// [`indexes`]: crate::parser::streamed::StreamedParserExt::indexes
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Indexes<P, const N: usize> {
    inner: P,
    ns: [usize; N],
}

impl<P, const N: usize> Indexes<P, N> {
    /// Creating a new instance.
    #[inline]
    pub fn new(inner: P, ns: [usize; N]) -> Self {
        Self { inner, ns }
    }

    /// Creating a new instance for method [`fill`].
    ///
    /// [`fill`]: crate::parser::streamed::StreamedParserExt::fill
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

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

crate::parser_state! {
    pub struct IndexesState<I, P: StreamedParser | const N: usize> {
        inner: P::State,
        count: usize,
        buf: UninitBuffer<P::Item, N>,
        end: bool,
        #[opt(set = set_start)]
        start: I::Locator,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, I, const N: usize> Parser<I> for Indexes<P, N>
where
    P: StreamedParser<I>,
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
                (Status::Success(Some(val), err), pos) if !state.buf.is_filled() => {
                    let index = self.ns[state.buf.index()];
                    if state.count == index {
                        state.buf.push(val);
                        if self.ns[state.buf.index()] <= index {
                            panic!("ns must be ascending ordered.");
                        }
                    }
                    state.count += 1;
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                }
                (Status::Success(Some(_), err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                }
                (Status::Success(None, err), pos) if state.buf.is_filled() => {
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start);
                    break (
                        Status::Success(Some(state.buf.take()), state.error()),
                        state.start()..pos.end,
                    );
                }
                (Status::Success(None, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    state.set_start(|| pos.start.clone());
                    break (Status::Success(None, state.error()), state.start()..pos.end);
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err), &pos);
                    state.set_start(|| pos.start);
                    break (
                        Status::Failure(state.error().unwrap(), false),
                        state.start()..pos.end,
                    );
                }
                (Status::Failure(err, true), pos) => {
                    state.set_start(|| pos.start);
                    break (Status::Failure(err, true), state.start()..pos.end);
                }
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
