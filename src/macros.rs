/// Call recursive parsers.
#[macro_export]
macro_rules! call {
    ($func:expr) => {{
        use $crate::parser::streamed::StreamedParserExt;
        use $crate::parser::ParserExt;
        $crate::parser::lazy(|| ($func)().no_state().boxed())
    }};
}

/// Automatically generate a [`Parser::State`] or [`StreamedParser::State`].
///
/// [`Parser::State`]: crate::parser::Parser::State
/// [`StreamedParser::State`]: crate::parser::streamed::StreamedParser::State
#[macro_export]
macro_rules! parser_state {
    (
        $(#[$attrs:meta])*
        $vis:vis struct $name:ident <
            $I:ident $(: $Itrait:path)?
            $(, $Ps:ident: $trait:ident)*
            $(; $($Ts:ident $(:$def:ident)?),*)? $(,)?
        > {
            $($(#[$opt:ident$(($($k:ident = $v:ident),*$(,)?))?])? $field:ident : $ty:ty),*$(,)?
        }
    ) => {
        $(#[$attrs])*
        $vis struct $name <
            $I: $crate::stream::Positioned $(+ $Itrait)? + ?Sized
            $(, $Ps: $trait<$I>)*
            $($(, $Ts)*)?
        > {
            $($field : $crate::parser_state_inner!{@type; $({$opt})? $ty}),*
        }

        impl <
            $I: $crate::stream::Positioned $(+ $Itrait)? + ?Sized
            $(, $Ps: $trait<$I>)*
            $($(, $Ts $(: core::default::$def)?)*)?
        > core::default::Default for $name <
            $I
            $(, $Ps)*
            $($(, $Ts)*)?
        > {
            #[inline]
            fn default() -> Self {
                Self {
                    $($field: core::default::Default::default()),*
                }
            }
        }

        impl <
            $I: $crate::stream::Positioned $(+ $Itrait)? + ?Sized
            $(, $Ps: $trait<$I>)*
            $($(, $Ts $(: core::default::$def)?)*)?
        > $name <
            $I
            $(, $Ps)*
            $($(, $Ts)*)?
        > {
            $($crate::parser_state_inner!{@impl; $({$($([$k = $v])*)?})? $field: $ty})*
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! parser_state_inner {
    (@type; $ty:ty) => {
        $ty
    };
    (@type; {opt} $ty:ty) => {
        core::option::Option<$ty>
    };
    (@impl; {} $field:ident : $ty:ty) => {
        #[allow(dead_code, non_snake_case)]
        fn $field(&mut self) -> $ty {
            core::mem::take(&mut self.$field).unwrap()
        }
    };
    (@impl; $field:ident : $ty:ty) => {
        #[allow(dead_code, non_snake_case)]
        fn $field(&mut self) -> $ty {
            core::mem::take(&mut self.$field)
        }
    };
}
