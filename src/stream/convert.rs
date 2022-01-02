//! Converting values while consuming a stream.

mod error;
mod stream;
pub use error::ConvertError;
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
    fn convert(&mut self, item: T, buf: &mut [Self::Output]) -> Result<usize, Self::Error>;
}
