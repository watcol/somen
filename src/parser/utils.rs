use core::mem;

use crate::error::Expects;

/// Merges two `Option<Expects<T>>` into one.
pub fn merge_expects<T>(this: &mut Option<Expects<T>>, other: Option<Expects<T>>) {
    *this = match (mem::take(this), other) {
        (Some(e), Some(f)) => Some(e.merge(f)),
        (this, other) => other.or(this),
    }
}
