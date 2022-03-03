use core::mem;
use core::ops::Range;

use crate::error::Error;

/// Merges two `Option<Error<T, L>>` into one.
pub fn merge_errors<T, L: PartialEq>(
    this: &mut Option<Error<T, L>>,
    other: Option<Error<T, L>>,
    pos: &Range<L>,
) {
    *this = if pos.start == pos.end {
        match (mem::take(this), other) {
            (Some(e), Some(f)) => Some(Error {
                expects: e.expects.merge(f.expects),
                position: e.position,
            }),
            (this, other) => other.or(this),
        }
    } else {
        other
    }
}
