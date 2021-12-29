//! ## TODO
//! - [x] UnpositionedStream (TryStream -> TryStream + Positioned)
//! - [x] InfallibleStream (Stream -> TryStream)
//! - [x] IteratorStream (Iterator -> Stream)
//! - [ ] PositionedStream (TryStream -> TryStream + Positioned)
//! - [ ] RecordedStream (TryStream -> TryStream + Rewind) (saving output with `Vec`, using it
//! to `Rewind`).
//! - [ ] ExtendStream (TryStream -> TryStream) (saving output with `Extend`)
//! - [ ] BufferedStream (TryStream -> TryStream + Rewind)
//! - [ ] ReaderStream (AsyncRead -> TryStream)
//! - [ ] SeekStream (TryStream + AsyncSeek -> TryStream + Rewind)

mod infallible;
mod iterator;
mod unpositioned;
pub use infallible::InfallibleStream;
pub use iterator::IteratorStream;
pub use unpositioned::UnpositionedStream;
