use core::ops::Range;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::streamed::StreamedParser;
use crate::parser::utils::merge_errors;
use crate::stream::Positioned;

macro_rules! tuple_parser {
    ($state:ident, $h:ident $(, $t:ident)*) => {
        crate::parser_state! {
            #[allow(non_snake_case)]
            pub struct $state <I, $h: StreamedParser, $($t: StreamedParser),* > {
                $h: ($h::State, bool),
                $(
                    $t: ($t::State, bool),
                )*
                error: Option<Error<I::Ok, I::Locator>>,
                #[opt]
                pos: Range<I::Locator>,
            }
        }

        impl<I, $h, $($t),*> StreamedParser<I> for ($h, $($t),*)
        where
            I: Positioned + ?Sized,
            $h: StreamedParser<I>,
            $( $t: StreamedParser<I, Item = $h::Item>, )*
        {
            type Item = $h::Item;
            type State = $state<I, $h, $($t),*>;

            fn poll_parse_next(
                &mut self,
                mut input: Pin<&mut I>,
                cx: &mut Context<'_>,
                state: &mut Self::State,
            ) -> PolledResult<Option<Self::Item>, I> {
                #[allow(non_snake_case)]
                let ($h, $($t),*) = self;

                if !state.$h.1 {
                    match ready!($h.poll_parse_next(input.as_mut(), cx, &mut state.$h.0)?) {
                        (Status::Success(None, err), pos) => {
                            state.$h.1 = true;
                            state.error = err;
                            state.pos = Some(pos);
                        }
                        res => return Poll::Ready(Ok(res)),
                    }
                }

                $(
                    if !state.$t.1 {
                        match ready!($t.poll_parse_next(input.as_mut(), cx, &mut state.$t.0)?) {
                            (Status::Success(Some(val), err), pos) if state.pos.is_some() => {
                                merge_errors(&mut state.error, err);
                                return Poll::Ready(Ok((
                                    Status::Success(Some(val), state.error()),
                                    state.pos().start..pos.end,
                                )));
                            }
                            (Status::Failure(err, false), pos) if state.pos.is_some() => {
                                merge_errors(&mut state.error, Some(err));
                                return Poll::Ready(Ok((
                                    Status::Failure(state.error().unwrap(), false),
                                    state.pos().start..pos.end,
                                )));
                            }
                            (Status::Success(None, err), pos) => {
                                state.$t.1 = true;
                                merge_errors(&mut state.error, err);
                                state.pos = Some(if state.pos.is_some() {
                                    state.pos().start..pos.end
                                } else {
                                    pos
                                });
                            }
                            res => return Poll::Ready(Ok(res)),
                        }
                    }
                )*

                Poll::Ready(Ok((
                    Status::Success(None, state.error()),
                    state.pos()
                )))
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                #[allow(non_snake_case)]
                let ($h, $($t),*) = self;
                #[allow(non_snake_case)]
                let ($h, $($t),*) = ($h.size_hint(), $($t.size_hint()),*);
                let min = $h.0 $(+ $t.0)*;
                let max = $h.1 $(.zip($t.1).map(|(a, b)| a + b))*;
                (min, max)
            }
        }
    };
}

tuple_parser! { State1, T1 }
tuple_parser! { State2, T1, T2 }
tuple_parser! { State3, T1, T2, T3 }
tuple_parser! { State4, T1, T2, T3, T4 }
tuple_parser! { State5, T1, T2, T3, T4, T5 }
tuple_parser! { State6, T1, T2, T3, T4, T5, T6 }
tuple_parser! { State7, T1, T2, T3, T4, T5, T6, T7 }
tuple_parser! { State8, T1, T2, T3, T4, T5, T6, T7, T8 }
tuple_parser! { State9, T1, T2, T3, T4, T5, T6, T7, T8, T9 }
tuple_parser! { State10, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10 }
// tuple_parser! { State11, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11 }
// tuple_parser! { State12, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12 }
// tuple_parser! { State13, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13 }
// tuple_parser! { State14, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14 }
// tuple_parser! { State15, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15 }
// tuple_parser! { State16, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16 }
// tuple_parser! { State17, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17 }
// tuple_parser! { State18, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18 }
// tuple_parser! { State19, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19 }
// tuple_parser! { State20, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20 }
