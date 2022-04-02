//! An asynchronous parser combinator library.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(doc_cfg))]
#![cfg_attr(feature = "nightly", feature(doc_notable_trait))]
#![cfg_attr(feature = "nightly", feature(try_trait_v2, try_trait_v2_residual))]
#![cfg_attr(feature = "nightly", feature(extend_one))]
#![doc(test(attr(warn(warnings))))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod parser;
pub mod stream;

mod macros;

/// Re-exports of commonly used items.
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::parser::iterable::IterableParserExt as _;
    #[doc(no_inline)]
    pub use crate::parser::ParserExt as _;
    #[doc(no_inline)]
    pub use crate::stream::StreamBuilder as _;

    pub use crate::parser::iterable::choice_iterable;
    pub use crate::parser::iterable::IterableParser;
    pub use crate::parser::Parser;
    pub use crate::parser::{
        any, choice, eof, function, is, is_not, is_some, lazy, none_of, not, one_of, position, tag,
        token, tokens, value, value_fn,
    };
    pub use crate::stream::{self, Input, Positioned};
}
