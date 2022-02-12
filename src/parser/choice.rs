use super::{Or, Parser};
use crate::stream::Input;

/// A helper trait for function [`choice`].
///
/// [`choice`]: crate::parser::choice
pub trait ChoiceParser<I: Input + ?Sized> {
    type Parser: Parser<I>;

    fn into_parser(self) -> Self::Parser;
}

macro_rules! choice_tuple {
    ($h:ident) => {
        impl<I, $h> ChoiceParser<I> for ($h,)
        where
            I: Input + ?Sized,
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
