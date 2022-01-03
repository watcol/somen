//! Converting values while consuming a stream.

mod error;
mod recorder;
pub use error::ConvertError;
pub use recorder::ConvertedRecorder;

#[cfg(feature = "alloc")]
mod stream;
#[cfg(feature = "alloc")]
pub use stream::ConvertedStream;

/// Converting a value into another type.
pub trait Converter<T> {
    /// The output type.
    type Output;
    /// The error type.
    type Error;

    /// Converting an item to [`Output`]s, storing to `buf`.
    ///
    /// [`Output`]: crate::stream::convert::Converter::Output
    fn convert<E: Extend<Self::Output>>(
        &mut self,
        item: T,
        buf: &mut E,
    ) -> Result<usize, Self::Error>;
}
