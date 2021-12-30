use super::{Positioned, Rewind, Unpositioned};

mod extend;
mod infallible;
mod iterator;
mod positioned;
mod unpositioned;
pub use extend::ExtendStream;
pub use infallible::InfallibleStream;
pub use iterator::IteratorStream;
pub use positioned::PositionedStream;
pub use unpositioned::UnpositionedStream;

#[cfg(feature = "alloc")]
mod buffered;
#[cfg(feature = "alloc")]
mod record;
#[cfg(feature = "alloc")]
pub use buffered::{BufferedError, BufferedStream};
#[cfg(feature = "alloc")]
pub use record::RecordStream;

#[cfg(feature = "std")]
mod reader;
#[cfg(feature = "std")]
mod seek;
#[cfg(feature = "std")]
pub use reader::ReaderStream;
#[cfg(feature = "std")]
pub use seek::{SeekError, SeekStream};
