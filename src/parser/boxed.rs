use core::marker::PhantomData;

use super::Parser;
use crate::error::{ParseError, ParseResult};
use crate::stream::position::Positioned;

use core::future::Future;
use futures_core::future::BoxFuture;

use alloc::boxed::Box;

/// The boxed parsers.
pub type BoxParser<'a, I, O, E, F> = Box<dyn Parser<I, Output = O, Error = E, Future = F> + 'a>;

impl<I: Positioned + ?Sized, O, E, F> Parser<I> for BoxParser<'_, I, O, E, F>
where
    F: Future<Output = Result<O, ParseError<E, I::Error, I::Position>>>,
{
    type Output = O;
    type Error = E;
    type Future = F;

    #[inline]
    fn parse(&self, input: &mut I) -> Self::Future {
        (**self).parse(input)
    }
}

/// A wrapper for parsers to box future objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FutureBoxed<'a, P> {
    parser: P,
    _phantom_a: PhantomData<&'a ()>,
}

impl<P> FutureBoxed<'_, P> {
    /// Creating a new instance.
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            _phantom_a: PhantomData,
        }
    }
}

impl<'a, P: Parser<I>, I: Positioned + ?Sized> Parser<I> for FutureBoxed<'a, P>
where
    P::Future: Send + 'a,
{
    type Output = P::Output;
    type Error = P::Error;
    type Future = BoxFuture<'a, ParseResult<Self, I>>;

    #[inline]
    fn parse(&self, input: &mut I) -> Self::Future {
        Box::pin(self.parser.parse(input))
    }
}
