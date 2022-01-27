//! Recording stream outputs.

mod extend;
mod vec;

use core::pin::Pin;

use alloc::borrow::{Cow, ToOwned};
pub use extend::ExtendRecorder;
pub use vec::VecRecorder;

/// Recording *consumed* items.
///
/// > Note: This trait is to record items consumed by the stream itself, so [`rewind`]ed items must
/// > not be duplicated.
///
/// [`rewind`]: super::rewind::Rewind::rewind
pub trait Record {
    /// The returned type with dereferenced format.
    type Borrowed: ToOwned + ?Sized;

    /// Start the recording.
    fn start(self: Pin<&mut Self>);

    /// Stop the recording, and returns a slice or an owned collection for the recorded items by
    /// [`Cow`].
    fn end(self: Pin<&mut Self>) -> Option<Cow<'_, Self::Borrowed>>;
}
