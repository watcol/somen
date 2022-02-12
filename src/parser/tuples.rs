use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use super::Parser;
use crate::error::{ParseResult, Tracker};
use crate::stream::Positioned;

macro_rules! tuple_parser {
    ($state:ident, $h:ident $(, $t:ident)*) => {
        #[derive(Debug)]
        #[allow(non_snake_case)]
        pub struct $state <I: Positioned + ?Sized, $h: Parser<I>, $( $t: Parser<I> ),* > {
            $h: (Option<$h::Output>, $h::State),
            $(
                $t: (Option<$t::Output>, $t::State),
            )*
        }

        impl<I, $h, $($t),*> Default for $state<I, $h, $($t),*>
        where I: Positioned + ?Sized,
              $h: Parser<I>,
              $( $t: Parser<I>, )*
        {
            #[inline]
            fn default() -> Self {
                Self {
                    $h: (None, Default::default()),
                    $( $t: (None, Default::default()), )*
                }
            }
        }

        impl<I, $h, $($t),*> Parser<I> for ($h, $($t),*)
        where
            I: Positioned + ?Sized,
            $h: Parser<I>,
            $( $t: Parser<I>, )*
        {
            type Output = ($h::Output, $( $t::Output ),*);
            type State = $state<I, $h, $($t),*>;

            fn poll_parse(
                &mut self,
                mut input: Pin<&mut I>,
                cx: &mut Context<'_>,
                state: &mut Self::State,
                tracker: &mut Tracker<I::Ok>,
            ) -> Poll<ParseResult<Self::Output, I>> {
                #[allow(non_snake_case)]
                let ($h, $($t),*) = self;

                if state.$h.0.is_none() {
                    state.$h.0 = Some(
                        ready!($h.poll_parse(input.as_mut(), cx, &mut state.$h.1, tracker))?
                    );
                }

                $(
                    if state.$t.0.is_none() {
                        state.$t.0 = Some(
                            ready!($t.poll_parse(input.as_mut(), cx, &mut state.$t.1, tracker))
                                .map_err(|err| err.fatal(true))?
                        );
                    }
                )*

                Poll::Ready(Ok((
                    mem::take(&mut state.$h.0).unwrap(),
                    $( mem::take(&mut state.$t.0).unwrap() ),*
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
tuple_parser! { State17, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17 }
tuple_parser! { State18, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18 }
tuple_parser! { State19, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19 }
tuple_parser! { State20, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20 }
