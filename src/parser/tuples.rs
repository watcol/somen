use core::fmt;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;

use super::Parser;
use crate::error::ParseResult;
use crate::stream::Positioned;

macro_rules! tuple_parser {
    ($error:ident, $state:ident $(, $t:ident )+) => {
        #[derive(Debug)]
        #[allow(non_snake_case)]
        pub enum $error < $($t),+ > {
            $(
                $t($t),
            )+
        }

        impl< $( $t: fmt::Display ),+ > fmt::Display for $error<$($t),+> {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $(
                        Self::$t(err) => err.fmt(f),
                    )+
                }
            }
        }

        #[cfg(feature = "std")]
        impl< $($t),+ > std::error::Error for $error<$($t),+>
        where
            $( $t: std::error::Error + 'static ),+
        {
            #[inline]
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    $(
                        Self::$t(err) => Some(err),
                    )+
                }
            }
        }

        #[derive(Debug)]
        #[allow(non_snake_case)]
        pub struct $state <I: Positioned + ?Sized, $( $t: Parser<I> ),+ > {
            $(
                $t: (Option<$t::Output>, $t::State),
            )+
        }

        impl<I, $($t),+> Default for $state<I, $($t),+>
        where I: Positioned + ?Sized,
              $( $t: Parser<I> ),+
        {
            #[inline]
            fn default() -> Self {
                Self {
                    $( $t: (None, Default::default()), )+
                }
            }
        }

        impl<I, $($t),+> Parser<I> for ($($t),+,)
        where
            I: Positioned + ?Sized,
            $( $t: Parser<I> ),+
        {
            type Output = ($( $t::Output ),+,);
            type Error = $error<$($t::Error),+>;
            type State = $state<I, $($t),+>;

            fn poll_parse(
                &mut self,
                mut input: Pin<&mut I>,
                cx: &mut Context<'_>,
                state: &mut Self::State,
            ) -> Poll<ParseResult<Self, I>> {
                #[allow(non_snake_case)]
                let ($(ref mut $t),+,) = *self;
                $(
                    if state.$t.0.is_none() {
                        state.$t.0 = Some(
                            ready!($t.poll_parse(input.as_mut(), cx, &mut state.$t.1))
                                .map_err(|err| err.map_parse($error::$t))?
                        );
                    }
                )+

                Poll::Ready(Ok((
                    $( mem::take(&mut state.$t.0).unwrap() ),+,
                )))
            }
        }
    };
}

tuple_parser! { Error1, State1, T1 }
tuple_parser! { Error2, State2, T1, T2 }
tuple_parser! { Error3, State3, T1, T2, T3 }
tuple_parser! { Error4, State4, T1, T2, T3, T4 }
tuple_parser! { Error5, State5, T1, T2, T3, T4, T5 }
tuple_parser! { Error6, State6, T1, T2, T3, T4, T5, T6 }
tuple_parser! { Error7, State7, T1, T2, T3, T4, T5, T6, T7 }
tuple_parser! { Error8, State8, T1, T2, T3, T4, T5, T6, T7, T8 }
tuple_parser! { Error9, State9, T1, T2, T3, T4, T5, T6, T7, T8, T9 }
tuple_parser! { Error10, State10, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10 }
tuple_parser! { Error11, State11, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11 }
tuple_parser! { Error12, State12, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12 }
tuple_parser! { Error13, State13, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13 }
tuple_parser! { Error14, State14, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14 }
tuple_parser! { Error15, State15, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15 }
tuple_parser! { Error16, State16, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16 }
tuple_parser! { Error17, State17, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17 }
tuple_parser! { Error18, State18, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18 }
tuple_parser! { Error19, State19, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19 }
tuple_parser! { Error20, State20, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20 }
