//! An asynchronous parser combinator library.

#![cfg_attr(all(doc, feature = "unstable"), feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod stream;
