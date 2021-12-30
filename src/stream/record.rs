//! Recording stream outputs.

mod extend;
pub use extend::ExtendRecorder;
#[cfg(feature = "alloc")]
mod vec;
#[cfg(feature = "alloc")]
pub use vec::VecRecorder;
