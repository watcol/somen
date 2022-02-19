use super::StreamedParser;
use crate::parser::Or;
use crate::stream::{Input, Positioned};

/// A helper trait for function [`choice_streamed`].
///
/// [`choice_streamed`]: crate::parser::streamed::choice_streamed
pub trait ChoiceStreamedParser<I: Positioned + ?Sized> {
    type Parser: StreamedParser<I>;

    fn into_streamed_parser(self) -> Self::Parser;
}

macro_rules! choice_tuple {
    ($h:ident) => {
        impl<I, $h> ChoiceStreamedParser<I> for ($h,)
        where
            I: Positioned + ?Sized,
            $h: StreamedParser<I>,
        {
            type Parser = $h;

            #[inline]
            fn into_streamed_parser(
                self,
            ) -> Self::Parser {
                self.0
            }
        }
    };

    ($h:ident $(, $t:ident)+) => {
        impl<I, $h, $($t),*> ChoiceStreamedParser<I> for ($h, $($t),*)
        where
            I: Input + ?Sized,
            $h: StreamedParser<I>,
            $( $t: StreamedParser<I, Item = $h::Item>, )+
        {
            type Parser = Or<$h, <($($t),+,) as ChoiceStreamedParser<I>>::Parser>;

            #[inline]
            fn into_streamed_parser(
                self,
            ) -> Self::Parser {
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
