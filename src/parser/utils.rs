use core::mem;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EitherState<C, D> {
    Left(C),
    Right(D),
}

impl<C: Default, D> Default for EitherState<C, D> {
    #[inline]
    fn default() -> Self {
        Self::Left(Default::default())
    }
}

impl<C, D> EitherState<C, D> {
    pub fn as_mut_left(&mut self) -> &mut C {
        match self {
            Self::Left(left) => left,
            Self::Right(_) => unreachable!(),
        }
    }

    pub fn as_mut_right(&mut self) -> &mut D {
        match self {
            Self::Left(_) => unreachable!(),
            Self::Right(right) => right,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpanState<C, L> {
    pub inner: C,
    pub start: Option<L>,
}

impl<C: Default, L> Default for SpanState<C, L> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
            start: None,
        }
    }
}

impl<C, L> SpanState<C, L> {
    pub fn set_start(&mut self, f: impl FnOnce() -> L) {
        if self.start.is_none() {
            self.start = Some(f())
        }
    }

    pub fn take_start(&mut self) -> L {
        mem::take(&mut self.start).unwrap()
    }
}
