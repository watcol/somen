/// Derived from unstable `core::iter::Step`.
pub trait Step: Clone + PartialOrd + Sized {
    fn steps_between(start: &Self, end: &Self) -> Option<usize>;
    fn forward(start: Self, count: usize) -> Option<Self>;
    fn backward(start: Self, count: usize) -> Option<Self>;
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct NopPosition;

impl Step for NopPosition {
    #[inline]
    fn steps_between(_: &Self, _: &Self) -> Option<usize> {
        Some(0)
    }

    #[inline]
    fn forward(_: Self, _: usize) -> Option<Self> {
        Some(NopPosition)
    }

    #[inline]
    fn backward(_: Self, _: usize) -> Option<Self> {
        Some(NopPosition)
    }
}
