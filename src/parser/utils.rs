use core::mem;

use crate::error::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EitherState<C, D> {
    Left(C),
    Right(D),
}

impl<C: Default, D> Default for EitherState<C, D> {
    #[inline]
    fn default() -> Self {
        Self::new_left()
    }
}

impl<C, D> EitherState<C, D> {
    #[inline]
    pub fn new_left() -> Self
    where
        C: Default,
    {
        Self::Left(Default::default())
    }

    #[inline]
    pub fn new_right() -> Self
    where
        D: Default,
    {
        Self::Right(Default::default())
    }

    #[inline]
    pub fn left(&mut self) -> &mut C {
        match self {
            Self::Left(left) => left,
            Self::Right(_) => unreachable!(),
        }
    }

    #[inline]
    pub fn right(&mut self) -> &mut D {
        match self {
            Self::Left(_) => unreachable!(),
            Self::Right(right) => right,
        }
    }
}

/// Merges two `Option<Error<L>>` into one.
pub fn merge_errors<L: PartialEq>(this: &mut Option<Error<L>>, other: Option<Error<L>>) {
    *this = match (mem::take(this), other) {
        (Some(e), Some(f)) if e.position.start == f.position.start => Some(Error {
            expects: e.expects.merge(f.expects),
            position: e.position,
        }),
        (this, other) => other.or(this),
    }
}
