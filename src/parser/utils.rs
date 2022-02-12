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
