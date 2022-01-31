use core::mem;
use core::ops::Range;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Expects, ParseError, ParseResult};
use crate::parser::Parser;
use crate::stream::Input;

/// A parser for method [`or`].
///
/// [`or`]: super::ParserExt::or
#[derive(Debug)]
pub struct Or<P, Q> {
    left: P,
    right: Q,
}

impl<P, Q> Or<P, Q> {
    /// Creating a new instance.
    #[inline]
    pub fn new(left: P, right: Q) -> Self {
        Self { left, right }
    }

    /// Extracting the inner parser.
    #[inline]
    pub fn into_inner(self) -> (P, Q) {
        (self.left, self.right)
    }
}

#[derive(Debug)]
enum EitherState<C, D> {
    Left(C),
    Right(D),
}

#[derive(Debug)]
pub struct OrState<C, D, T, L, M> {
    inner: EitherState<C, D>,
    left_error: Option<(Expects<T>, Range<L>)>,
    queued_marker: Option<M>,
}

impl<C: Default, D, T, L, M> Default for OrState<C, D, T, L, M> {
    fn default() -> Self {
        Self {
            inner: EitherState::Left(C::default()),
            left_error: None,
            queued_marker: None,
        }
    }
}

impl<P, Q, I> Parser<I> for Or<P, Q>
where
    P: Parser<I>,
    Q: Parser<I, Output = P::Output>,
    I: Input + ?Sized,
{
    type Output = P::Output;
    #[allow(clippy::type_complexity)]
    type State = OrState<P::State, Q::State, I::Ok, I::Locator, I::Marker>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> Poll<ParseResult<Self::Output, I>> {
        if let EitherState::Left(ref mut inner) = state.inner {
            if state.queued_marker.is_none() {
                state.queued_marker = Some(input.as_mut().mark()?);
            }

            match ready!(self.left.poll_parse(input.as_mut(), cx, inner)) {
                Ok(i) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    return Poll::Ready(Ok(i));
                }
                Err(ParseError::Parser(ex, p)) => {
                    input
                        .as_mut()
                        .rewind(mem::take(&mut state.queued_marker).unwrap())?;
                    state.left_error = Some((ex, p));
                    state.inner = EitherState::Right(Default::default());
                }
                Err(err) => {
                    input.drop_marker(mem::take(&mut state.queued_marker).unwrap())?;
                    return Poll::Ready(Err(err));
                }
            }
        }

        if let EitherState::Right(ref mut inner) = state.inner {
            self.right
                .poll_parse(input, cx, inner)
                .map_err(|err| match err {
                    ParseError::Parser(ex, p) => {
                        let (ex2, p2) = mem::take(&mut state.left_error).unwrap();
                        if p.start == p2.start {
                            ParseError::Parser(ex.merge(ex2), p.start..(p.end.max(p2.end)))
                        } else {
                            ParseError::Parser(ex, p)
                        }
                    }
                    e => e,
                })
        } else {
            unreachable!()
        }
    }
}
