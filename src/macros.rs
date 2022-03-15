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
            $(; $($Ts:ident $(:$def:ident)?),*)?
            $(| $(const $Cs:ident : $C_ty:ty),*)? $(,)?
        > {
            $($(#[$opt:ident$(($($k:ident = $v:ident),*$(,)?))?])? $field:ident : $ty:ty),*$(,)?
        }
    ) => {
        $(#[$attrs])*
        $vis struct $name <
            $I: $crate::stream::Positioned $(+ $Itrait)? + ?Sized
            $(, $Ps: $trait<$I>)*
            $($(, $Ts)*)?
            $($(, const $Cs: $C_ty)*)?
        > {
            $($field : $crate::parser_state_inner!{@type; $({$opt})? $ty}),*
        }

        impl <
            $I: $crate::stream::Positioned $(+ $Itrait)? + ?Sized
            $(, $Ps: $trait<$I>)*
            $($(, $Ts $(: core::default::$def)?)*)?
            $($(, const $Cs: $C_ty)*)?
        > core::default::Default for $name <
            $I
            $(, $Ps)*
            $($(, $Ts)*)?
            $($(, $Cs)*)?
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
            $($(, const $Cs: $C_ty)*)?
        > $name <
            $I
            $(, $Ps)*
            $($(, $Ts)*)?
            $($(, $Cs)*)?
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
    (@impl; $field:ident : $ty:ty) => {
        #[allow(dead_code, non_snake_case)]
        #[inline]
        fn $field(&mut self) -> $ty {
            core::mem::take(&mut self.$field)
        }
    };
    (@impl; {} $field:ident : $ty:ty) => {
        #[allow(dead_code, non_snake_case)]
        #[inline]
        fn $field(&mut self) -> $ty {
            core::mem::take(&mut self.$field).unwrap()
        }
    };
    (@impl; {[set = $func:ident]$([$key:ident = $value:ident])*} $field:ident : $ty:ty) => {
        #[inline]
        fn $func<F: FnOnce() -> $ty>(&mut self, f: F) {
            if self.$field.is_none() {
                self.$field = Some(f());
            }
        }

        $crate::parser_state_inner! {@impl; {$([$key = $value])*} $field: $ty}
    };
    (@impl; {[try_set = $func:ident]$([$key:ident = $value:ident])*} $field:ident : $ty:ty) => {
        #[cfg(feature = "nightly")]
        #[inline]
        fn $func<F: FnOnce() -> R, R: core::ops::Try<Output = $ty>>(&mut self, f: F)
            -> <R::Residual as core::ops::Residual<()>>::TryType
        where R::Residual: core::ops::Residual<()>,
        {
            if self.$field.is_none() {
                self.$field = Some(f()?);
            }
            core::ops::Try::from_output(())
        }

        #[cfg(not(feature = "nightly"))]
        #[inline]
        fn $func<F: FnOnce() -> core::result::Result<$ty, E>, E>(&mut self, f: F)
            -> core::result::Result<(), E>
        {
            if self.$field.is_none() {
                self.$field = Some(f()?);
            }
            core::result::Result::Ok(())
        }

        $crate::parser_state_inner! {@impl; {$([$key = $value])*} $field: $ty}
    };
    (@impl; {[get = $func:ident]$([$key:ident = $value:ident])*} $field:ident : $ty:ty) => {
        #[inline]
        fn $func(&self) -> &$ty {
            self.$field.as_ref().unwrap()
        }

        $crate::parser_state_inner! {@impl; {$([$key = $value])*} $field: $ty}
    };
    (@impl; {[get_mut = $func:ident]$([$key:ident = $value:ident])*} $field:ident : $ty:ty) => {
        #[inline]
        fn $func(&mut self) -> &mut $ty {
            self.$field.as_mut().unwrap()
        }

        $crate::parser_state_inner! {@impl; {$([$key = $value])*} $field: $ty}
    };
    (@impl; {[clear = $func:ident]$([$key:ident = $value:ident])*} $field:ident : $ty:ty) => {
        #[inline]
        fn $func(&mut self) {
            self.$field = None;
        }

        $crate::parser_state_inner! {@impl; {$([$key = $value])*} $field: $ty}
    };
    (@impl; {[$k:ident = $v:ident]$([$key:ident = $value:ident])*} $field:ident : $ty:ty) => {
        $crate::parser_state_inner! {@impl; {$([$key = $value])*} $field: $ty}
    };
}
