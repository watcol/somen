//! ## TODO
//! - [x] UnpositionedStream (TryStream -> TryStream + Positioned)
//! - [x] InfallibleStream (Stream -> TryStream)
//! - [x] IteratorStream (Iterator -> Stream)
//! - [x] PositionedStream (TryStream -> TryStream + Positioned)
//! - [ ] RecordStream (TryStream -> TryStream + Rewind) (saving output with `Vec`, using it
//! to `Rewind`).
//! - [ ] ExtendStream (TryStream -> TryStream) (saving output with `Extend`)
//! - [ ] BufferedStream (TryStream -> TryStream + Rewind)
//! - [ ] ReaderStream (AsyncRead -> TryStream)
//! - [ ] SeekStream (TryStream + AsyncSeek -> TryStream + Rewind)

use super::{Positioned, Rewind, Unpositioned};

mod infallible;
mod iterator;
mod positioned;
mod unpositioned;
pub use infallible::InfallibleStream;
pub use iterator::IteratorStream;
pub use positioned::PositionedStream;
pub use unpositioned::UnpositionedStream;

#[cfg(feature = "alloc")]
mod record;
#[cfg(feature = "alloc")]
pub use record::RecordStream;
