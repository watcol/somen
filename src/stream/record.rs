//! Recording stream outputs.

mod extend;
#[cfg(feature = "alloc")]
mod vec;

pub use extend::ExtendRecorder;
#[cfg(feature = "alloc")]
pub use vec::VecRecorder;
