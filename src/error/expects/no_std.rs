use core::fmt;

pub type Expect = &'static str;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expects(Expect);

impl From<Expect> for Expects {
    #[inline]
    fn from(inner: Expect) -> Self {
        Self(inner)
    }
}

impl fmt::Display for Expects {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl IntoIterator for Expects {
    type Item = Expect;
    type IntoIter = core::iter::Once<Expect>;

    fn into_iter(self) -> Self::IntoIter {
        core::iter::once(self.0)
    }
}

impl FromIterator<Expect> for Expects {
    fn from_iter<I: IntoIterator<Item = Expect>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        match iter.next() {
            Some(ex) if iter.next().is_none() => Self(ex),
            _ => Self("something"),
        }
    }
}

impl Expects {
    #[inline]
    #[allow(unused_variables)]
    pub fn merge(self, other: Self) -> Self {
        Self("something")
    }

    #[inline]
    pub fn map<F: FnMut(Expect) -> Expect>(self, mut f: F) -> Self {
        Expects(f(self.0))
    }
}
