#[derive(Debug)]
pub enum EitherState<C, D> {
    Left(C),
    Right(D),
}

impl<C: Default, D> Default for EitherState<C, D> {
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
