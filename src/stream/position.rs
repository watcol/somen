//! Positioning streams.

mod positioned;
pub use positioned::PositionedStream;

mod locator;
pub use locator::{LineCol, Locator};

use futures_core::TryStream;

/// A stream that records its position.
///
/// this trait will be used in so many situations (e.g. returning the position
/// where errors have occured), so all streams should implement this.
pub trait Positioned: TryStream {
    /// The type of the position.
    type Locator: Locator<Self::Ok>;

    /// Getting the current position.
    fn position(&self) -> Self::Locator;
}
