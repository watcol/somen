use core::cmp::Ordering;

/// A trait for types represents positions for streams of `T`s.
pub trait Locator<T: ?Sized> {
    /// Incrementing the position to the next token, by referencing the consumed token.
    fn next(&mut self, token: &T);
}

impl<T: ?Sized> Locator<T> for () {
    fn next(&mut self, _token: &T) {}
}

macro_rules! locator_impl {
    ($t:ty) => {
        impl<T: ?Sized> Locator<T> for $t {
            fn next(&mut self, _token: &T) {
                *self += 1;
            }
        }
    };
}

locator_impl! { i8 }
locator_impl! { i16 }
locator_impl! { i32 }
locator_impl! { i64 }
locator_impl! { i128 }
locator_impl! { isize }
locator_impl! { u8 }
locator_impl! { u16 }
locator_impl! { u32 }
locator_impl! { u64 }
locator_impl! { u128 }
locator_impl! { usize }

/// A locator for streams of [`char`]s, indicates line and column index.
///
/// The index starts with `1`, and only `\n` will be treated as a newline character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

impl Default for LineCol {
    #[inline]
    fn default() -> Self {
        Self { line: 1, col: 1 }
    }
}

impl Ord for LineCol {
    fn cmp(&self, other: &Self) -> Ordering {
        let line_ord = self.line.cmp(&other.line);
        if line_ord != Ordering::Equal {
            line_ord
        } else {
            self.col.cmp(&other.col)
        }
    }
}

impl PartialOrd for LineCol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Locator<char> for LineCol {
    fn next(&mut self, token: &char) {
        if *token == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }
}
