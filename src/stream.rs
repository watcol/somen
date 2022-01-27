//! Streams used as the input of parsers.
//!
//! The input of parser should implement [`TryStream`] and [`Positioned`] defined here (and
//! sometimes [`Rewind`] will be required), so here we'll provide some implementations
//! on them by wrapping types implementing [`Stream`], [`AsyncRead`], etc.
//!
//! [`Stream`]: futures_core::stream::Stream
//! [`TryStream`]: futures_core::stream::TryStream
//! [`AsyncRead`]: futures_io::AsyncRead

mod builder;
mod imp;
pub use builder::*;
pub use imp::*;

pub mod position;
#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
pub mod record;
pub mod rewind;

pub use position::Positioned;
#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
pub use record::Record;
pub use rewind::Rewind;

/// An alias trait for [`Positioned`]` + `[`Rewind`].
pub trait Input: Positioned + Rewind {}

impl<T: Positioned + Rewind + ?Sized> Input for T {}

/// An alias trait for [`Positioned`]` + `[`Record`].
#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
pub trait NoRewindInput: Positioned + Record {}

#[cfg(feature = "alloc")]
impl<T: Positioned + Record + ?Sized> NoRewindInput for T {}

/// An alias trait for [`Positioned`]` + `[`Rewind`]` + `[`Record`].
#[cfg(feature = "alloc")]
#[cfg_attr(feature = "nightly", doc(cfg(feature = "alloc")))]
pub trait RecordInput: Positioned + Rewind + Record {}

#[cfg(feature = "alloc")]
impl<T: Positioned + Rewind + Record + ?Sized> RecordInput for T {}
