/// Wrapping a function returns a parser to generate a recursive parser.
#[macro_export]
macro_rules! call {
    ($func:expr) => {
        $crate::parser::function(|input, cx, state| ($func)().poll_parse(input, cx, state))
    };
}
