use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::iterable::IterableParser;
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
    /// Creates a new instance.
    #[inline]
    pub fn new(left: P, right: Q) -> Self {
        Self { left, right }
    }

    /// Extracts the inner parser.
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
        #[opt]
        start: I::Locator,
        error: Option<Error<I::Locator>>,
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

            if state.start.is_none() {
                state.start = Some(input.position());
            }

            match ready!(self.left.poll_parse(input.as_mut(), cx, inner)?) {
                Status::Success(val, err) => {
                    input.drop_marker(state.marker())?;
                    return Poll::Ready(Ok(Status::Success(val, err)));
                }
                Status::Failure(err, false) if err.rewindable(&state.start()) => {
                    input.as_mut().rewind(state.marker())?;
                    state.error = Some(err);
                    state.inner = EitherState::new_right();
                }
                Status::Failure(err, exclusive) => {
                    input.drop_marker(state.marker())?;
                    return Poll::Ready(Ok(Status::Failure(err, exclusive)));
                }
            }
        }

        self.right
            .poll_parse(input, cx, state.inner.right())
            .map_ok(|status| match status {
                Status::Success(val, err) => {
                    merge_errors(&mut state.error, err);
                    Status::Success(val, state.error())
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    Status::Failure(state.error().unwrap(), false)
                }
                res => res,
            })
    }
}

crate::parser_state! {
    pub struct OrIterableState<I: Input, P: IterableParser, Q: IterableParser> {
        inner: EitherState<P::State, Q::State>,
        succeeded: bool,
        #[opt]
        marker: I::Marker,
        #[opt]
        start: I::Locator,
        error: Option<Error<I::Locator>>,
    }
}

impl<P, Q, I> IterableParser<I> for Or<P, Q>
where
    P: IterableParser<I>,
    Q: IterableParser<I, Item = P::Item>,
    I: Input + ?Sized,
{
    type Item = P::Item;
    type State = OrIterableState<I, P, Q>;

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

            if state.start.is_none() {
                state.start = Some(input.position());
            }

            match ready!(self.left.poll_parse_next(input.as_mut(), cx, inner)?) {
                Status::Success(val, err) => {
                    input.drop_marker(state.marker())?;
                    state.succeeded = true;
                    return Poll::Ready(Ok(Status::Success(val, err)));
                }
                Status::Failure(err, false) if err.rewindable(&state.start()) => {
                    input.as_mut().rewind(state.marker())?;
                    state.error = Some(err);
                    state.inner = EitherState::new_right();
                }
                Status::Failure(err, exclusive) => {
                    input.drop_marker(state.marker())?;
                    return Poll::Ready(Ok(Status::Failure(err, exclusive)));
                }
            }
        }

        if state.succeeded {
            return self.right.poll_parse_next(input, cx, state.inner.right());
        }

        self.right
            .poll_parse_next(input, cx, state.inner.right())
            .map_ok(|status| match status {
                Status::Success(val, err) => {
                    state.succeeded = true;
                    merge_errors(&mut state.error, err);
                    Status::Success(val, state.error())
                }
                Status::Failure(err, false) => {
                    merge_errors(&mut state.error, Some(err));
                    Status::Failure(state.error().unwrap(), false)
                }
                res => res,
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

/// A helper trait for function [`choice_iterable`].
///
/// [`choice_iterable`]: crate::parser::iterable::choice_iterable
pub trait ChoiceIterableParser<I: Positioned + ?Sized> {
    /// The output parser type.
    type IterableParser: IterableParser<I>;

    /// Generate a choice parser from tuples.
    fn into_iterable_parser(self) -> Self::IterableParser;
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

        impl<I, $h> ChoiceIterableParser<I> for ($h,)
        where
            I: Positioned + ?Sized,
            $h: IterableParser<I>,
        {
            type IterableParser = $h;

            #[inline]
            fn into_iterable_parser(
                self,
            ) -> Self::IterableParser {
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

        impl<I, $h, $($t),*> ChoiceIterableParser<I> for ($h, $($t),*)
        where
            I: Input + ?Sized,
            $h: IterableParser<I>,
            $( $t: IterableParser<I, Item = $h::Item>, )+
        {
            type IterableParser = Or<$h, <($($t),+,) as ChoiceIterableParser<I>>::IterableParser>;

            #[inline]
            fn into_iterable_parser(
                self,
            ) -> Self::IterableParser {
                #[allow(non_snake_case)]
                let ($h, $($t),+) = self;
                Or::new($h, ($($t),+,).into_iterable_parser())
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
