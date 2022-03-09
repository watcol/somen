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
            $($field : $crate::parser_state_inner!{@type; $($({$k $(= $v)?})*)? $ty}),*
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
                    $($field: $crate::parser_state_inner!{@default; $($({$k $(= $v)?})*)? $ty}),*
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
            $($crate::parser_state_inner!{@impl; $($({$k $(= $v)?})*)? $field: $ty})*
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! parser_state_inner {
    (@type; $ty:ty) => {
        $ty
    };
    (@type; {option}$({$k_rest:ident $(= $v_rest:ident)?})* $ty:ty) => {
        core::option::Option<$ty>
    };
    (@type; {$k:ident $(= $v:ident)?}$({$k_rest:ident $(= $v_rest:ident)?})* $ty:ty) => {
        $crate::parser_state_inner!(@type; $({$k_rest $(= $v_rest)?})* $ty)
    };
    (@default; .opt $ty:ty) => {
        core::option::Option::None
    };
    (@default; $ty:ty) => {
        core::default::Default::default()
    };
    (@default;
       $(.opt)? {default = $func:ident}$({$k_rest:ident $(= $v_rest:ident)?})* $ty:ty
    ) => {
        $func()
    };
    (@default;
        $(.opt)? {option}$({$k_rest:ident $(= $v_rest:ident)?})* $ty:ty
    ) => {
        $crate::parser_state_inner!{@default; .opt $({$k_rest $(= $v_rest)?})* $ty}
    };
    (@default;
        $(.$opt:ident)? {$k:ident $(= $v:ident)?}$({$k_rest:ident $(= $v_rest:ident)?})* $ty:ty
    ) => {
        $crate::parser_state_inner!{@default; $(.$opt)? $({$k_rest = $v_rest})* $ty}
    };
    (@impl;
        <opt = opt> $(<set = $set:ident>)? $(<get = $get:ident>)? $(<take = $take:ident>)?
        $field:ident : $ty:ty
    ) => {
        $(
            #[inline]
            fn $set<F: Fn() -> $ty>(&mut self, f: F) {
                if self.$field.is_none() {
                    self.$field = Some(f());
                }
            }
        )?
        $(
            #[inline]
            fn $get(&self) -> $ty {
                self.$field.clone().unwrap()
            }
        )?
        $(
            #[inline]
            fn $take(&mut self) -> $ty {
                core::mem::take(&mut self.$field).unwrap()
            }
        )?
    };
    (@impl;
        $(<set = $set:ident>)? $(<get = $get:ident>)? $(<take = $take:ident>)?
        $field:ident : $ty:ty
    ) => {
        $(
            #[inline]
            fn $set<F: Fn() -> $ty>(&mut self, f: F) {
                self.$field = f();
            }
        )?
        $(
            #[inline]
            fn $get(&self) -> $ty {
                self.$field.clone()
            }
        )?
        $(
            #[inline]
            fn $take(&mut self) -> $ty {
                core::mem::take(&mut self.$field)
            }
        )?
    };
    (@impl;
        $(<opt = $_:ident>)? $(<set = $set:ident>)? $(<get = $get:ident>)? $(<take = $take:ident>)?
        {option}$({$k_rest:ident $(= $v_rest:ident)?})* $field:ident : $ty:ty
    ) => {
        $crate::parser_state_inner!{@impl;
            <opt = opt> $(<set = $set>)? $(<get = $get>)? $(<take = $take>)?
            $({$k_rest $(= $v_rest)?})* $field: $ty
        }
    };
    (@impl;
        $(<opt = $opt:ident>)? $(<set = $_:ident>)? $(<get = $get:ident>)? $(<take = $take:ident>)?
        {set = $set:ident}$({$k_rest:ident $(= $v_rest:ident)?})* $field:ident : $ty:ty
    ) => {
        $crate::parser_state_inner!{@impl;
            $(<opt = $opt>)? <set = $set> $(<get = $get>)? $(<take = $take>)?
            $({$k_rest $(= $v_rest)?})* $field: $ty
        }
    };
    (@impl;
        $(<opt = $opt:ident>)? $(<set = $set:ident>)? $(<get = $_:ident>)? $(<take = $take:ident>)?
        {get = $get:ident}$({$k_rest:ident $(= $v_rest:ident)?})* $field:ident : $ty:ty
    ) => {
        $crate::parser_state_inner!{@impl;
            $(<opt = $opt>)? $(<set = $set>)? <get = $get> $(<take = $take>)?
            $({$k_rest $(= $v_rest)?})* $field: $ty
        }
    };
    (@impl;
        $(<opt = $opt:ident>)? $(<set = $set:ident>)? $(<get = $get:ident>)? $(<take = $_:ident>)?
        {take = $take:ident}$({$k_rest:ident $(= $v_rest:ident)?})* $field:ident : $ty:ty
    ) => {
        $crate::parser_state_inner!{@impl;
            $(<opt = $opt>)? $(<set = $set>)? $(<get = $get>)? <take = $take>
            $({$k_rest $(= $v_rest)?})* $field: $ty
        }
    };
    (@impl;
        $(<opt = $opt:ident>)? $(<set = $set:ident>)? $(<get = $get:ident>)? $(<take = $take:ident>)?
        {$k:ident $(= $v:ident)?}$({$k_rest:ident $(= $v_rest:ident)?})* $field:ident : $ty:ty
    ) => {
        $crate::parser_state_inner!{@impl;
            $(<opt = $opt>)? $(<set = $set>)? $(<get = $get>)? $(<take = $take>)?
            $({$k_rest $(= $v_rest)?})* $field: $ty
        }
    };
}
