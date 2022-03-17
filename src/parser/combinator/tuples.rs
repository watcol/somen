use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use crate::error::{Error, PolledResult, Status};
use crate::parser::utils::merge_errors;
use crate::parser::Parser;
use crate::stream::Positioned;

macro_rules! tuple_parser {
    ($state:ident $(, $t:ident)*) => {
        $crate::parser_state! {
            #[allow(non_snake_case)]
            pub struct $state <I, $( $t: Parser ),*> {
                $( $t: (Option<$t::Output>, $t::State), )*
                error: Option<Error<I::Ok, I::Locator>>,
            }
        }

        impl<I, $($t),*> Parser<I> for ($($t),*,)
        where
            I: Positioned + ?Sized,
            $( $t: Parser<I>, )*
        {
            type Output = ($( $t::Output ),*,);
            type State = $state<I, $($t),*>;

            fn poll_parse(
                &mut self,
                mut input: Pin<&mut I>,
                cx: &mut Context<'_>,
                state: &mut Self::State,
            ) -> PolledResult<Self::Output, I> {
                #[allow(non_snake_case)]
                let ($($t),*,) = self;

                $(
                    if state.$t.0.is_none() {
                        match ready!($t.poll_parse(input.as_mut(), cx, &mut state.$t.1)?) {
                            Status::Success(val, err) => {
                                state.$t.0 = Some(val);
                                merge_errors(&mut state.error, err);
                            }
                            Status::Failure(err, false) => {
                                merge_errors(&mut state.error, Some(err));
                                return Poll::Ready(Ok(
                                    Status::Failure(state.error().unwrap(), false)
                                ));
                            },
                            Status::Failure(err, true) => {
                                return Poll::Ready(Ok(Status::Failure(err, true)));
                            }
                        }
                    }
                )*

                Poll::Ready(Ok(Status::Success(
                    ($(mem::take(&mut state.$t.0).unwrap()),*,),
                    state.error()
                )))
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
tuple_parser! { State11, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11 }
tuple_parser! { State12, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12 }
tuple_parser! { State13, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13 }
tuple_parser! { State14, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14 }
tuple_parser! { State15, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15 }
tuple_parser! { State16, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16 }
