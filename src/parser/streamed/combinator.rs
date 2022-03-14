//! Streamed parser combinators.
mod collect;
mod count;
mod discard;
mod last;
mod nth;
mod tuples;

pub use collect::Collect;
pub use count::Count;
pub use discard::Discard;
pub use last::Last;
pub use nth::Nth;
