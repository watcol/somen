mod infallible;
mod iterator;
mod slice;
pub use infallible::InfallibleStream;
pub use iterator::IteratorStream;
pub use slice::SliceStream;

#[cfg(feature = "std")]
mod reader;
#[cfg(feature = "std")]
pub use reader::ReaderStream;
