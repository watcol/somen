use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::{merge_errors, EitherState};
use crate::parser::Parser;
use crate::stream::{Input, Positioned};

/// A parser for method [`or`].
///
/// [`or`]: crate::parser::ParserExt::or
#[derive(Clone, Debug, PartialEq, Eq)]
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

crate::parser_state! {
    pub struct OrState<I: Input, P: Parser, Q: Parser> {
        inner: EitherState<P::State, Q::State>,
        #[opt]
        marker: I::Marker,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> Parser<I> for Or<P, Q>
where
    P: Parser<I>,
    Q: Parser<I, Output = P::Output>,
    I: Input + ?Sized,
{
    type Output = P::Output;
    type State = OrState<I, P, Q>;

    fn poll_parse(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Self::Output, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            if state.marker.is_none() {
                state.marker = Some(input.as_mut().mark()?);
            }

            match ready!(self.left.poll_parse(input.as_mut(), cx, inner)?) {
                (Status::Success(i, err), pos) => {
                    input.drop_marker(state.marker())?;
                    return Poll::Ready(Ok((Status::Success(i, err), pos)));
                }
                (Status::Failure(err, false), pos) if err.rewindable(&pos.start) => {
                    input.as_mut().rewind(state.marker())?;
                    state.error = Some(err);
                    state.inner = EitherState::new_right();
                }
                (Status::Failure(err, exclusive), pos) => {
                    input.drop_marker(state.marker())?;
                    return Poll::Ready(Ok((Status::Failure(err, exclusive), pos)));
                }
            }
        }

        self.right
            .poll_parse(input, cx, state.inner.right())
            .map_ok(|status| match status {
                (Status::Success(i, err), pos) => {
                    merge_errors(&mut state.error, err, &pos);
                    (Status::Success(i, state.error()), pos)
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err), &pos);
                    (Status::Failure(state.error().unwrap(), false), pos)
                }
                res @ (Status::Failure(_, true), _) => res,
            })
    }
}

crate::parser_state! {
    pub struct OrStreamedState<I: Input, P: StreamedParser, Q: StreamedParser> {
        inner: EitherState<P::State, Q::State>,
        succeeded: bool,
        #[opt]
        marker: I::Marker,
        error: Option<Error<I::Ok, I::Locator>>,
    }
}

impl<P, Q, I> StreamedParser<I> for Or<P, Q>
where
    P: StreamedParser<I>,
    Q: StreamedParser<I, Item = P::Item>,
    I: Input + ?Sized,
{
    type Item = P::Item;
    type State = OrStreamedState<I, P, Q>;

    fn poll_parse_next(
        &mut self,
        mut input: Pin<&mut I>,
        cx: &mut Context<'_>,
        state: &mut Self::State,
    ) -> PolledResult<Option<Self::Item>, I> {
        if let EitherState::Left(inner) = &mut state.inner {
            if state.succeeded {
                return self.left.poll_parse_next(input, cx, inner);
            }

            if state.marker.is_none() {
                state.marker = Some(input.as_mut().mark()?);
            }

            match ready!(self.left.poll_parse_next(input.as_mut(), cx, inner)?) {
                (Status::Success(i, err), pos) => {
                    input.drop_marker(state.marker())?;
                    state.succeeded = true;
                    return Poll::Ready(Ok((Status::Success(i, err), pos)));
                }
                (Status::Failure(err, false), pos) if err.rewindable(&pos.start) => {
                    input.as_mut().rewind(state.marker())?;
                    state.error = Some(err);
                    state.inner = EitherState::new_right();
                }
                (Status::Failure(err, exclusive), pos) => {
                    input.drop_marker(state.marker())?;
                    return Poll::Ready(Ok((Status::Failure(err, exclusive), pos)));
                }
            }
        }

        if state.succeeded {
            return self.right.poll_parse_next(input, cx, state.inner.right());
        }

        self.right
            .poll_parse_next(input, cx, state.inner.right())
            .map_ok(|status| match status {
                (Status::Success(i, err), pos) => {
                    state.succeeded = true;
                    merge_errors(&mut state.error, err, &pos);
                    (Status::Success(i, state.error()), pos)
                }
                (Status::Failure(err, false), pos) => {
                    merge_errors(&mut state.error, Some(err), &pos);
                    (Status::Failure(state.error().unwrap(), false), pos)
                }
                res @ (Status::Failure(_, true), _) => res,
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lmin, lmax) = self.left.size_hint();
        let (rmin, rmax) = self.right.size_hint();
        (
            core::cmp::min(lmin, rmin),
            lmax.zip(rmax).map(|(a, b)| core::cmp::max(a, b)),
        )
    }
}

/// A helper trait for function [`choice`].
///
/// [`choice`]: crate::parser::choice
pub trait ChoiceParser<I: Positioned + ?Sized> {
    /// The output parser type.
    type Parser: Parser<I>;

    /// Generate a choice parser from tuples.
    fn into_parser(self) -> Self::Parser;
}

/// A helper trait for function [`choice_streamed`].
///
/// [`choice_streamed`]: crate::parser::streamed::choice_streamed
pub trait ChoiceStreamedParser<I: Positioned + ?Sized> {
    /// The output parser type.
    type StreamedParser: StreamedParser<I>;

    /// Generate a choice parser from tuples.
    fn into_streamed_parser(self) -> Self::StreamedParser;
}

macro_rules! choice_tuple {
    ($h:ident) => {
        impl<I, $h> ChoiceParser<I> for ($h,)
        where
            I: Positioned + ?Sized,
            $h: Parser<I>,
        {
            type Parser = $h;

            #[inline]
            fn into_parser(
                self,
            ) -> Self::Parser {
                self.0
            }
        }

        impl<I, $h> ChoiceStreamedParser<I> for ($h,)
        where
            I: Positioned + ?Sized,
            $h: StreamedParser<I>,
        {
            type StreamedParser = $h;

            #[inline]
            fn into_streamed_parser(
                self,
            ) -> Self::StreamedParser {
                self.0
            }
        }
    };

    ($h:ident $(, $t:ident)+) => {
        impl<I, $h, $($t),*> ChoiceParser<I> for ($h, $($t),*)
        where
            I: Input + ?Sized,
            $h: Parser<I>,
            $( $t: Parser<I, Output = $h::Output>, )+
        {
            type Parser = Or<$h, <($($t),+,) as ChoiceParser<I>>::Parser>;

            #[inline]
            fn into_parser(
                self,
            ) -> Self::Parser {
                #[allow(non_snake_case)]
                let ($h, $($t),+) = self;
                Or::new($h, ($($t),+,).into_parser())
            }
        }

        impl<I, $h, $($t),*> ChoiceStreamedParser<I> for ($h, $($t),*)
        where
            I: Input + ?Sized,
            $h: StreamedParser<I>,
            $( $t: StreamedParser<I, Item = $h::Item>, )+
        {
            type StreamedParser = Or<$h, <($($t),+,) as ChoiceStreamedParser<I>>::StreamedParser>;

            #[inline]
            fn into_streamed_parser(
                self,
            ) -> Self::StreamedParser {
                #[allow(non_snake_case)]
                let ($h, $($t),+) = self;
                Or::new($h, ($($t),+,).into_streamed_parser())
            }
        }
    };
}

choice_tuple! { T1 }
choice_tuple! { T1, T2 }
choice_tuple! { T1, T2, T3 }
choice_tuple! { T1, T2, T3, T4 }
choice_tuple! { T1, T2, T3, T4, T5 }
choice_tuple! { T1, T2, T3, T4, T5, T6 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7, T8 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9, T10 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19 }
choice_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20 }