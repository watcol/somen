//! An asynchronous parser combinator library.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(doc_cfg))]
#![cfg_attr(feature = "nightly", feature(doc_notable_trait))]
#![cfg_attr(feature = "nightly", feature(try_trait_v2, try_trait_v2_residual))]
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
    pub use crate::parser::streamed::StreamedParserExt as _;
    #[doc(no_inline)]
    pub use crate::parser::ParserExt as _;
    #[doc(no_inline)]
    pub use crate::stream::StreamBuilder as _;

    pub use crate::parser::streamed::StreamedParser;
    pub use crate::parser::Parser;
    pub use crate::stream::{self, Input, Positioned};
}
