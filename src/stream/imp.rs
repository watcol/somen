mod infallible;
mod iterator;
#[cfg(feature = "std")]
mod reader;
mod slice;

pub use infallible::InfallibleStream;
pub use iterator::IteratorStream;
#[cfg(feature = "std")]
pub use reader::ReaderStream;
pub use slice::SliceStream;
