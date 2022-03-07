/// Call recursive parsers.
#[macro_export]
macro_rules! call {
    ($func:expr) => {{
        use $crate::parser::streamed::StreamedParserExt;
        use $crate::parser::ParserExt;
        $crate::parser::lazy(|| ($func)().no_state().boxed())
    }};
}

/// Automatically generate a [`Parser::State`] or [`StreamedParser::State`]
///
/// [`Parser`]: crate::parser::Parser
/// [`StreamedParser`]: crate::parser::streamed::StreamedParser
#[macro_export]
macro_rules! parser_state {
    (
        $(#[$attrs:meta])*
        $vis:vis struct $name:ident <
            $I:ident $(: $Itrait:path)?
            $(, $Ps:ident: $trait:ident)*
            $(; $($Ts:ident $(:$def:ident)?),*)? $(,)?
        > {
            $($field:ident : $ty:ty),*$(,)?
        }
    ) => {
        $(#[$attrs])*
        #[derive(Clone, Debug)]
        $vis struct $name <
            $I: $crate::stream::Positioned $(+ $Itrait)? + ?Sized
            $(, $Ps: $trait<$I>)*
            $($(, $Ts)*)?
        > {
            $($field : $ty),*
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
    }
}
