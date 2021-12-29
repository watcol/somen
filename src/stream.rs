//! Streams used as the input of parsers.
//!
//! The input of parser should implement [`TryStream`] and [`Positioned`] defined here (and
//! sometimes [`Rewind`] will be required), so here we'll provide some implementations
//! on them by wrapping types implementing [`Stream`], [`AsyncRead`], etc.
//!
//! ## TODO
//! - [ ] InfallibleStream (Stream -> TryStream)
//! - [ ] IteratorStream (Iterator -> Stream)
//! - [ ] PositionedStream (TryStream -> TryStream + Positioned)
//! - [ ] RecordedStream (TryStream -> TryStream + Rewind) (saving output with `Vec`, using it
//! to `Rewind`).
//! - [ ] ExtendStream (TryStream -> TryStream) (saving output with `Extend`)
//! - [ ] BufferedStream (TryStream -> TryStream + Rewind)
//! - [ ] ReaderStream (AsyncRead -> TryStream)
//! - [ ] SeekStream (TryStream + AsyncSeek -> TryStream + Rewind)
//!
//! [`Rewind`]: ./trait.Rewind.html
//! [`Positioned`]: ./trait.Positioned.html
//! [`Stream`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
//! [`TryStream`]: https://docs.rs/futures/latest/futures/stream/trait.TryStream.html
//! [`AsyncRead`]: https://docs.rs/futures/latest/futures/io/trait.AsyncRead.html

mod position;
mod rewind;

pub use position::Positioned;
pub use rewind::Rewind;
