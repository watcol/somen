//! An asynchronous parser combinator library.
//!
//! flumen (/ˈfluː.men/, "river" or "stream" in Latin) is an asynchronous parser combinator.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod stream;
