use core::marker::PhantomData;

use super::{Parser, PositionedParser};
use crate::error::{ParseError, ParseResult, PositionedResult};
use crate::stream::position::Positioned;
use crate::stream::BasicInput;

use core::future::Future;
use futures_core::future::BoxFuture;

use alloc::boxed::Box;

/// The boxed parsers.
pub type BoxParser<'a, I, O, E, F> = Box<dyn Parser<I, Output = O, Error = E, Future = F> + 'a>;

impl<I: BasicInput + ?Sized, O, E, F> Parser<I> for BoxParser<'_, I, O, E, F>
where
    F: Future<Output = Result<O, ParseError<E, I::Error>>>,
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
pub struct FutureBoxed<'a, 'b, P> {
    parser: P,
    _phantom_a: PhantomData<&'a ()>,
    _phantom_b: PhantomData<&'b ()>,
}

impl<P> FutureBoxed<'_, '_, P> {
    /// Creating a new instance.
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            _phantom_a: PhantomData,
            _phantom_b: PhantomData,
        }
    }
}

impl<'a, P: Parser<I>, I: BasicInput + ?Sized> Parser<I> for FutureBoxed<'a, '_, P>
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

impl<'a, 'b, P: PositionedParser<I>, I> PositionedParser<I> for FutureBoxed<'a, 'b, P>
where
    I: BasicInput + Positioned + ?Sized,
    <P as Parser<I>>::Future: Send + 'a,
    <P as PositionedParser<I>>::Future: Send + 'b,
{
    type Future = BoxFuture<'b, PositionedResult<Self, I>>;

    #[inline]
    fn parse_positioned(&self, input: &mut I) -> <Self as PositionedParser<I>>::Future {
        Box::pin(self.parser.parse_positioned(input))
    }
}
