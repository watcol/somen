use alloc::borrow::Cow;
use alloc::collections::BTreeSet;
use alloc::string::String;
use core::fmt;

/// A expected token.
pub type Expect = Cow<'static, str>;

/// A set of expected tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expects(BTreeSet<Expect>);

impl From<Expect> for Expects {
    #[inline]
    fn from(msg: Cow<'static, str>) -> Self {
        Self(BTreeSet::from([msg]))
    }
}

impl From<&'static str> for Expects {
    #[inline]
    fn from(msg: &'static str) -> Self {
        Self(BTreeSet::from([Expect::from(msg)]))
    }
}

impl From<String> for Expects {
    #[inline]
    fn from(msg: String) -> Self {
        Self(BTreeSet::from([Expect::from(msg)]))
    }
}

impl fmt::Display for Expects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.0.len();
        if len == 0 {
            Ok(())
        } else if len == 1 {
            write!(f, "{}", self.0.iter().next().unwrap())
        } else {
            for (c, i) in self.0.iter().enumerate() {
                if c == 0 {
                    write!(f, "one of {}", i)?;
                } else if c == len - 1 {
                    write!(f, ", or {}", i)?;
                } else {
                    write!(f, ", {}", i)?;
                }
            }
            Ok(())
        }
    }
}

impl FromIterator<Expect> for Expects {
    fn from_iter<I: IntoIterator<Item = Expect>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl FromIterator<&'static str> for Expects {
    fn from_iter<I: IntoIterator<Item = &'static str>>(iter: I) -> Self {
        Self(iter.into_iter().map(Expect::from).collect())
    }
}

impl FromIterator<String> for Expects {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self(iter.into_iter().map(Expect::from).collect())
    }
}

impl IntoIterator for Expects {
    type Item = Expect;
    type IntoIter = alloc::collections::btree_set::IntoIter<Expect>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Expects {
    /// Merges two sets.
    pub fn merge(mut self, mut other: Expects) -> Self {
        self.0.append(&mut other.0);
        self
    }

    /// Converts each elements.
    pub fn map<F: FnMut(Expect) -> Expect>(self, mut f: F) -> Self {
        Expects(self.into_iter().map(&mut f).collect())
    }
}
