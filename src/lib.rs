//! An asynchronous parser combinator library.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(doc_cfg))]
#![doc(test(attr(warn(warnings))))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
mod macros;
pub mod parser;
pub mod stream;

/// Re-exports of commonly used items.
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::stream::StreamBuilder as _;

    pub use crate::parser::streamed::StreamedParser;
    pub use crate::parser::{any, function, Parser};
    pub use crate::stream::{self, Input, Positioned};
}
