use core::future::Future;
use core::marker::PhantomData;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::{ready, TryStream};

use crate::parser::Parser;
use crate::stream::Positioned;

use super::StreamedParser;

/// A parser collecting outputs of [`StreamedParser`].
#[derive(Debug)]
pub struct Collected<P, E> {
    parser: P,
    _phantom: PhantomData<E>,
}

impl<P, E> Collected<P, E> {
    /// Creating a new instance.
    #[inline]
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            _phantom: PhantomData,
        }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> P {
        self.parser
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct CollectedFuture<S, E> {
        #[pin]
        stream: S,
        collection: E,
    }
}

impl<S, E> Future for CollectedFuture<S, E>
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

impl<'parser, 'input, P, E, I> Parser<'parser, 'input, I> for Collected<P, E>
where
    P: StreamedParser<'parser, 'input, I>,
    E: Default + Extend<P::Output>,
    I: Positioned + ?Sized,
{
    type Output = E;
    type Error = P::Error;
    type Future = CollectedFuture<P::Stream, E>;

    fn parse(&'parser self, input: &'input mut I) -> Self::Future {
        CollectedFuture {
            stream: self.parser.parse_streamed(input),
            collection: E::default(),
        }
    }
}
