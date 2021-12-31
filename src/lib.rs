//! An asynchronous parser combinator library.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(doc, feature = "unstable"), feature(doc_cfg))]
#![cfg_attr(doc, doc(test(attr(warn(warnings)))))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod stream;
