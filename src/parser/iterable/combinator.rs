//! Iterable parser combinators.
mod collect;
mod count;
mod discard;
mod enumerate;
mod filter;
mod flatten;
mod fold;
mod indexes;
mod last;
mod nth;
mod reduce;
mod scan;
mod tuples;

pub use collect::Collect;
pub use count::Count;
pub use discard::Discard;
pub use enumerate::Enumerate;
pub use filter::Filter;
pub use flatten::Flatten;
pub use fold::{Fold, TryFold};
pub use indexes::Indexes;
pub use last::Last;
pub use nth::Nth;
pub use reduce::{Reduce, TryReduce};
pub use scan::{Scan, TryScan};
