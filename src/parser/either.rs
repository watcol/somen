use core::pin::Pin;
use core::task::{Context, Poll};

use crate::error::ParseResult;
use crate::parser::Parser;
use crate::stream::Positioned;

/// A parser for method [`left`], [`right`].
///
/// [`left`]: super::ParserExt::left
/// [`right`]: super::ParserExt::right
#[derive(Debug)]
pub enum Either<T, U> {
    Left(T),
    Right(U),
}

impl<T, U> Either<T, U> {
    fn unwrap_left(&mut self) -> &mut T {
        match self {
            Self::Left(left) => left,
            Self::Right(_) => unreachable!(),
        }
    }

    fn unwrap_right(&mut self) -> &mut U {
        match self {
            Self::Left(_) => unreachable!(),
            Self::Right(right) => right,
        }
    }
}

impl<P, Q, I> Parser<I> for Either<P, Q>
where
    P: Parser<I>,
    Q: Parser<I, Output = P::Output>,
    I: Positioned + ?Sized,
{
    type Output = P::Output;
    type State = Option<Either<P::State, Q::State>>;

    fn poll_parse(
        &mut self,
        input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self::Output, I>> {
        match self {
            Self::Left(left) => {
                if !matches!(state, Some(Either::Left(_))) {
                    *state = Some(Either::Left(Default::default()));
                }
                left.poll_parse(input, cx, state.as_mut().unwrap().unwrap_left())
            }
            Self::Right(right) => {
                if !matches!(state, Some(Either::Right(_))) {
                    *state = Some(Either::Right(Default::default()));
                }
                right.poll_parse(input, cx, state.as_mut().unwrap().unwrap_right())
            }
        }
    }
}
