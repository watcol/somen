use core::future::Future;
use core::marker::PhantomData;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, FusedFuture, FusedStream, TryStream};

use crate::parser::Parser;
use crate::stream::Positioned;

use super::StreamedParser;

/// A parser collecting outputs from a [`StreamedParser`].
#[derive(Debug)]
pub struct Collect<P, E> {
    inner: P,
    _phantom: PhantomData<E>,
}

impl<P, E> Collect<P, E> {
    /// Creating a new instance.
    #[inline]
    pub fn new(parser: P) -> Self {
        Self {
            inner: parser,
            _phantom: PhantomData,
        }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.inner
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct CollectFuture<S, E> {
        #[pin]
        stream: S,
        collection: E,
    }
}

impl<S, E> FusedFuture for CollectFuture<S, E>
where
    S: TryStream + FusedStream,
    E: Default + Extend<S::Ok>,
{
    #[inline]
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<S, E> Future for CollectFuture<S, E>
where
    S: TryStream,
    E: Default + Extend<S::Ok>,
{
    type Output = Result<E, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        loop {
            match ready!(this.stream.as_mut().try_poll_next(cx)?) {
                Some(x) => this.collection.extend([x]),
                None => break Poll::Ready(Ok(mem::take(this.collection))),
            }
        }
    }
}

impl<'parser, 'input, P, E, I> Parser<'parser, 'input, I> for Collect<P, E>
where
    P: StreamedParser<'parser, 'input, I>,
    E: Default + Extend<P::Output>,
    I: Positioned + ?Sized,
{
    type Output = E;
    type Error = P::Error;
    type Future = CollectFuture<P::Stream, E>;

    fn parse(&'parser self, input: &'input mut I) -> Self::Future {
        CollectFuture {
            stream: self.inner.parse_streamed(input),
            collection: E::default(),
        }
    }
}
