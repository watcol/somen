use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use super::StreamedParser;
use crate::error::{ParseResult, Tracker};
use crate::stream::Positioned;

macro_rules! tuple_parser {
    ($state:ident, $h:ident $(, $t:ident)*) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        #[allow(non_snake_case)]
        pub struct $state <$h, $($t),* > {
            $h: Option<$h>,
            $(
                $t: Option<$t>,
            )*
        }

        impl<$h, $($t),*> Default for $state<$h, $($t),*>
        where
            $h: Default,
            $( $t: Default, )*
        {
            #[inline]
            fn default() -> Self {
                Self {
                    $h: Some(Default::default()),
                    $( $t: Some(Default::default()), )*
                }
            }
        }

        impl<I, $h, $($t),*> StreamedParser<I> for ($h, $($t),*)
        where
            I: Positioned + ?Sized,
            $h: StreamedParser<I>,
            $( $t: StreamedParser<I, Item = $h::Item>, )*
        {
            type Item = $h::Item;
            type State = $state<$h::State, $($t::State),*>;

            fn poll_parse_next(
                &mut self,
                mut input: Pin<&mut I>,
                cx: &mut Context<'_>,
                state: &mut Self::State,
                tracker: &mut Tracker<I::Ok>,
            ) -> Poll<ParseResult<Option<Self::Item>, I>> {
                #[allow(non_snake_case)]
                let ($h, $($t),*) = self;

                if let Some(inner) = &mut state.$h {
                    match ready!($h.poll_parse_next(input.as_mut(), cx, inner, tracker))? {
                        Some(val) => return Poll::Ready(Ok(Some(val))),
                        None => state.$h = None,
                    }
                }

                $(
                    if let Some(inner) = &mut state.$t {
                        match
                            ready!($t.poll_parse_next(input.as_mut(), cx, inner, tracker))
                                .map_err(|err| err.fatal(true))?
                        {
                            Some(val) => return Poll::Ready(Ok(Some(val))),
                            None => state.$t = None,
                        }
                    }
                )*

                Poll::Ready(Ok(None))
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
