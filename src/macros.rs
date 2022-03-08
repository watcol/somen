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
            $($(#[state($($k:ident $(= $v:ident)?),*$(,)?)])? $field:ident : $ty:ty),*$(,)?
        }
    ) => {
        $(#[$attrs])*
        $vis struct $name <
            $I: $crate::stream::Positioned $(+ $Itrait)? + ?Sized
            $(, $Ps: $trait<$I>)*
            $($(, $Ts)*)?
        > {
            $($field : $crate::parser_state_inner!{@type; $($([$k $(= $v)?])*)? $ty}),*
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
                    $($field: $crate::parser_state_inner!{@default; $($([$k $(= $v)?])*)? $ty}),*
                }
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! parser_state_inner {
    (@type; $ty:ty) => {
        $ty
    };
    (@type; [option]$([$k_rest:ident $(= $v_rest:ident)?])* $ty:ty) => {
        core::option::Option<$ty>
    };
    (@type; [$k:ident $(= $v:ident)?]$([$k_rest:ident $(= $v_rest:ident)?])* $ty:ty) => {
        $crate::parser_state_inner!(@type; $([$k_rest $(= $v_rest)?])* $ty)
    };
    (@default; <opt> $ty:ty) => {
        core::option::Option::None
    };
    (@default; $ty:ty) => {
        core::default::Default::default()
    };
    (@default;
        [default = $func:ident]$([$k_rest:ident $(= $v_rest:ident)?])* $(<opt>)? $ty:ty
    ) => {
        $func()
    };
    (@default;
        [option]$([$k_rest:ident $(= $v_rest:ident)?])* $(<opt>)? $ty:ty
    ) => {
        $crate::parser_state_inner!{@default; $([$k_rest $(= $v_rest)?])* <opt> $ty}
    };
    (@default;
        [$k:ident $(= $v:ident)?]$([$k_rest:ident $(= $v_rest:ident)?])* $(<$opt:ident>)? $ty:ty
    ) => {
        $crate::parser_state_inner!{@default; $([$k_rest = $v_rest])* $(<$opt>)? $ty}
    };
}
