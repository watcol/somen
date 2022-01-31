#[macro_export]
macro_rules! call {
    ($func:expr) => {{
        use $crate::parser::ParserExt;
        $crate::parser::lazy(|| ($func)().no_state().boxed())
    }};
}
