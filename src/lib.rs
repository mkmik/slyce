//! Slyce implements a python-like slicer for rust.
//!
//! Indices can be addressed as absolute positions or relative to the end of the array (Tail).
//! Out of bound indices are ignored.
//!
//! Slice indices are represented with an enum that wraps the full `usize` range, but also
//! captures the possibility of a "negative" or "backward" index.
//! This crate provides a few implementations of `From<T> for Index` for common types,
//! so you can pass numbers and options instead of Index (just call `.into()`).
//!
//! # Example
//! ```
//! use slyce::{Slice, Index};
//! let v = vec![10,20,30,40,50];
//! let render = |s: Slice| format!("{:?}", s.apply(&v).collect::<Vec<_>>());
//!
//! let start: isize = -3;
//! let s = slyce::Slice{start: start.into(), end: Index::Default, step: None};
//! assert_eq!(render(s), "[30, 40, 50]");
//!
//! let s = slyce::Slice{start: Index::Tail(3), end: Index::Default, step: None};
//! assert_eq!(render(s), "[30, 40, 50]");
//!
//! let end: Option<isize> = None;
//! let s = slyce::Slice{start: Index::Tail(3), end: end.into(), step: None};
//! assert_eq!(render(s), "[30, 40, 50]");
//!
//! let s = slyce::Slice{start: Index::Tail(3), end: Index::Default, step: Some(-1)};
//! assert_eq!(render(s), "[30, 20, 10]");
//!
//! let s = slyce::Slice{start: Index::Head(4), end: Index::Head(0), step: Some(-1)};
//! assert_eq!(render(s), "[50, 40, 30, 20]");
//!
//! let s = slyce::Slice{start: Index::Default, end: Index::Head(0), step: Some(-1)};
//! assert_eq!(render(s), "[50, 40, 30, 20]");
//!
//! let s = slyce::Slice{start: Index::Tail(1000), end: 2000.into(), step: None};
//! assert_eq!(render(s), "[10, 20, 30, 40, 50]");
//! ```

use std::default::Default;
use std::fmt;
use std::ops::Range;

/// A slice has an optional start, an optional end, and an optional step.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Slice {
    pub start: Index,
    pub end: Index,
    pub step: Option<isize>,
}

impl fmt::Display for Slice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}:{}:{}]",
            self.start,
            self.end,
            self.step.map_or("".to_string(), |n| n.to_string())
        )
    }
}

/// A position inside an array.
///
/// Tail indices are represented with a distinct enumeration variant so that the full index
/// numeric range (usize) can be utilized without numeric overflows.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum Index {
    /// Position in the array relative to the start of the array (i.e. absolute position).
    Head(usize),
    /// Position in the array relative to the end of the array.
    Tail(usize),
    /// Either the first or the last element of the array, depending on the sign of `step`.
    Default,
}

use Index::*;

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Head(n) => write!(f, "{}", n),
            Tail(n) => write!(f, "-{}", n),
            Default => write!(f, ""),
        }
    }
}

impl Slice {
    /// Returns an iterator that yields the elements that match the slice expression.
    pub fn apply<'a, T>(&self, arr: &'a [T]) -> impl Iterator<Item = &'a T> + 'a {
        self.indices(arr.len()).map(move |i| &arr[i])
    }
    /// Returns an iterator that yields the indices that match the slice expression.
    fn indices(&self, len: usize) -> impl Iterator<Item = usize> {
        self.to_sslice().indices(len)
    }

    fn to_sslice(&self) -> SSlice {
        SSlice {
            start: self.start.to_signed(),
            end: self.end.to_signed(),
            step: self.step.map(|s| s as i128),
        }
    }
}

impl Index {
    fn to_signed(&self) -> Option<i128> {
        match self {
            &Head(n) => Some(n as i128),
            &Tail(n) => Some(-(n as i128)),
            Default => None,
        }
    }
}

/// A SSlice is a slice implemented using signed arithmetics. It uses a signed type that
/// must be able to represent usize. This impl won't work if size_of<usize> >= 16.
/// 64 bits of address space should be enough for everybody.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SSlice {
    start: Option<i128>,
    end: Option<i128>,
    step: Option<i128>,
}

impl SSlice {
    /// Returns an iterator that yields the indices that match the slice expression.
    fn indices(&self, len: usize) -> impl Iterator<Item = usize> {
        let ilen = len as i128;

        if len == 0 {
            return Iter {
                i: 0,
                end: 0,
                step: 0,
            };
        }

        let step = self.step.unwrap_or(1);

        let (def_start, def_end) = if step >= 0 {
            (0, ilen)
        } else {
            (ilen - 1, -ilen - 1)
        };

        let shift = if step >= 0 { 0 } else { 1 };
        let min = 0 - shift;
        let max = ilen - shift;

        let abs = |n: i128| if n >= 0 { n } else { ilen + n };
        Iter {
            i: clamp(abs(self.start.unwrap_or(def_start)), min..max),
            end: clamp(abs(self.end.unwrap_or(def_end)), min..max),
            step,
        }
    }
}

fn clamp<T: Ord>(n: T, r: Range<T>) -> T {
    r.start.max(n).min(r.end)
}

struct Iter {
    i: i128,
    end: i128,
    step: i128,
}

/// An iterator that counts from an initial number until a final limit.
/// The direction and stride of the iteration can be controlled by the step parameter.
/// A zero step produces an empty iteration.
impl Iterator for Iter {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.step == 0 {
            return None;
        }

        let is_in_range = if self.step >= 0 {
            |a, b| a < b
        } else {
            |a, b| a > b
        };
        let i = self.i;
        self.i += self.step;

        if is_in_range(i, self.end) {
            Some(i as usize)
        } else {
            None
        }
    }
}

impl From<usize> for Index {
    fn from(i: usize) -> Self {
        Head(i)
    }
}

impl From<isize> for Index {
    fn from(i: isize) -> Self {
        if i < 0 {
            Tail(-i as usize)
        } else {
            Head(i as usize)
        }
    }
}

impl From<i32> for Index {
    fn from(i: i32) -> Self {
        if i < 0 {
            Tail(-i as usize)
        } else {
            Head(i as usize)
        }
    }
}

impl<U> From<Option<U>> for Index
where
    U: Into<Index>,
{
    fn from(i: Option<U>) -> Self {
        i.map_or(Default, Into::into)
    }
}

impl Default for Index {
    fn default() -> Self {
        Index::Default
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn positive() {
        const LEN: usize = 4;

        fn s(start: Option<isize>, end: Option<isize>, step: Option<isize>) -> Vec<usize> {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }.indices(LEN).collect()
        }

        assert_eq!(s(None, None, None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(0), None, None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(1), None, None), vec![1, 2, 3]);
        assert_eq!(s(Some(2), None, None), vec![2, 3]);
        assert_eq!(s(Some(3), None, None), vec![3]);
        assert_eq!(s(Some(4), None, None), vec![]);
        assert_eq!(s(Some(5), None, None), vec![]);

        assert_eq!(s(Some(0), Some(4), None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(1), Some(4), None), vec![1, 2, 3]);
        assert_eq!(s(Some(2), Some(4), None), vec![2, 3]);
        assert_eq!(s(Some(3), Some(4), None), vec![3]);
        assert_eq!(s(Some(4), Some(4), None), vec![]);
        assert_eq!(s(Some(5), Some(4), None), vec![]);

        assert_eq!(s(Some(0), Some(3), None), vec![0, 1, 2]);
        assert_eq!(s(Some(1), Some(3), None), vec![1, 2]);
        assert_eq!(s(Some(2), Some(3), None), vec![2]);
        assert_eq!(s(Some(3), Some(3), None), vec![]);
        assert_eq!(s(Some(4), Some(3), None), vec![]);
        assert_eq!(s(Some(5), Some(3), None), vec![]);

        assert_eq!(s(Some(0), Some(2), None), vec![0, 1]);
        assert_eq!(s(Some(1), Some(2), None), vec![1]);
        assert_eq!(s(Some(2), Some(2), None), vec![]);
        assert_eq!(s(Some(3), Some(2), None), vec![]);
        assert_eq!(s(Some(4), Some(2), None), vec![]);
        assert_eq!(s(Some(5), Some(2), None), vec![]);

        assert_eq!(s(Some(0), Some(1), None), vec![0]);
        assert_eq!(s(Some(1), Some(1), None), vec![]);
        assert_eq!(s(Some(2), Some(1), None), vec![]);
        assert_eq!(s(Some(3), Some(1), None), vec![]);
        assert_eq!(s(Some(4), Some(1), None), vec![]);
        assert_eq!(s(Some(5), Some(1), None), vec![]);

        assert_eq!(s(Some(0), Some(0), None), vec![]);
        assert_eq!(s(Some(1), Some(0), None), vec![]);
        assert_eq!(s(Some(2), Some(0), None), vec![]);
        assert_eq!(s(Some(3), Some(0), None), vec![]);
        assert_eq!(s(Some(4), Some(0), None), vec![]);
        assert_eq!(s(Some(5), Some(0), None), vec![]);
    }

    #[test]
    fn negative_start() {
        const LEN: usize = 4;

        fn s(start: Option<isize>, end: Option<isize>, step: Option<isize>) -> Vec<usize> {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }.indices(LEN).collect()
        }

        assert_eq!(s(Some(-113667776004), None, None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(-6), None, None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(-5), None, None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(-4), None, None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(-3), None, None), vec![1, 2, 3]);
        assert_eq!(s(Some(-2), None, None), vec![2, 3]);
        assert_eq!(s(Some(-1), None, None), vec![3]);

        assert_eq!(s(Some(-5), Some(4), None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(-4), Some(4), None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(-3), Some(4), None), vec![1, 2, 3]);
        assert_eq!(s(Some(-2), Some(4), None), vec![2, 3]);
        assert_eq!(s(Some(-1), Some(4), None), vec![3]);

        assert_eq!(s(Some(-5), Some(3), None), vec![0, 1, 2]);
        assert_eq!(s(Some(-4), Some(3), None), vec![0, 1, 2]);
        assert_eq!(s(Some(-3), Some(3), None), vec![1, 2]);
        assert_eq!(s(Some(-2), Some(3), None), vec![2]);
        assert_eq!(s(Some(-1), Some(3), None), vec![]);

        assert_eq!(s(Some(-5), Some(2), None), vec![0, 1]);
        assert_eq!(s(Some(-4), Some(2), None), vec![0, 1]);
        assert_eq!(s(Some(-3), Some(2), None), vec![1]);
        assert_eq!(s(Some(-2), Some(2), None), vec![]);
        assert_eq!(s(Some(-1), Some(2), None), vec![]);

        assert_eq!(s(Some(-5), Some(1), None), vec![0]);
        assert_eq!(s(Some(-4), Some(1), None), vec![0]);
        assert_eq!(s(Some(-3), Some(1), None), vec![]);
        assert_eq!(s(Some(-2), Some(1), None), vec![]);
        assert_eq!(s(Some(-1), Some(1), None), vec![]);

        assert_eq!(s(Some(-5), Some(0), None), vec![]);
        assert_eq!(s(Some(-4), Some(0), None), vec![]);
        assert_eq!(s(Some(-3), Some(0), None), vec![]);
        assert_eq!(s(Some(-2), Some(0), None), vec![]);
        assert_eq!(s(Some(-1), Some(0), None), vec![]);
    }

    #[test]
    fn negative_end() {
        const LEN: usize = 4;

        fn s(start: Option<isize>, end: Option<isize>, step: Option<isize>) -> Vec<usize> {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }.indices(LEN).collect()
        }

        assert_eq!(s(None, None, None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(0), None, None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(1), None, None), vec![1, 2, 3]);
        assert_eq!(s(Some(2), None, None), vec![2, 3]);
        assert_eq!(s(Some(3), None, None), vec![3]);
        assert_eq!(s(Some(4), None, None), vec![]);
        assert_eq!(s(Some(5), None, None), vec![]);

        assert_eq!(s(Some(0), Some(4), None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(1), Some(4), None), vec![1, 2, 3]);
        assert_eq!(s(Some(2), Some(4), None), vec![2, 3]);
        assert_eq!(s(Some(3), Some(4), None), vec![3]);
        assert_eq!(s(Some(4), Some(4), None), vec![]);
        assert_eq!(s(Some(5), Some(4), None), vec![]);

        assert_eq!(s(Some(0), Some(-1), None), vec![0, 1, 2]);
        assert_eq!(s(Some(1), Some(-1), None), vec![1, 2]);
        assert_eq!(s(Some(2), Some(-1), None), vec![2]);
        assert_eq!(s(Some(3), Some(-1), None), vec![]);
        assert_eq!(s(Some(4), Some(-1), None), vec![]);
        assert_eq!(s(Some(5), Some(-1), None), vec![]);

        assert_eq!(s(Some(0), Some(-2), None), vec![0, 1]);
        assert_eq!(s(Some(1), Some(-2), None), vec![1]);
        assert_eq!(s(Some(2), Some(-2), None), vec![]);
        assert_eq!(s(Some(3), Some(-2), None), vec![]);
        assert_eq!(s(Some(4), Some(-2), None), vec![]);
        assert_eq!(s(Some(5), Some(-2), None), vec![]);

        assert_eq!(s(Some(0), Some(-3), None), vec![0]);
        assert_eq!(s(Some(1), Some(-3), None), vec![]);
        assert_eq!(s(Some(2), Some(-3), None), vec![]);
        assert_eq!(s(Some(3), Some(-3), None), vec![]);
        assert_eq!(s(Some(4), Some(-3), None), vec![]);
        assert_eq!(s(Some(5), Some(-3), None), vec![]);

        assert_eq!(s(Some(0), Some(-4), None), vec![]);
        assert_eq!(s(Some(1), Some(-4), None), vec![]);
        assert_eq!(s(Some(2), Some(-4), None), vec![]);
        assert_eq!(s(Some(3), Some(-4), None), vec![]);
        assert_eq!(s(Some(4), Some(-4), None), vec![]);
        assert_eq!(s(Some(5), Some(-4), None), vec![]);

        assert_eq!(s(Some(0), Some(-5), None), vec![]);
        assert_eq!(s(Some(1), Some(-5), None), vec![]);
        assert_eq!(s(Some(2), Some(-5), None), vec![]);
        assert_eq!(s(Some(3), Some(-5), None), vec![]);
        assert_eq!(s(Some(4), Some(-5), None), vec![]);
        assert_eq!(s(Some(5), Some(-5), None), vec![]);

        assert_eq!(s(Some(5), Some(-113667776004), None), vec![]);
    }

    #[test]
    fn oob() {
        const LEN: usize = 4;

        fn s(start: Option<isize>, end: Option<isize>, step: Option<isize>) -> Vec<usize> {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }.indices(LEN).collect()
        }

        assert_eq!(s(Some(0), Some(6), None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(1), Some(6), None), vec![1, 2, 3]);
        assert_eq!(s(Some(2), Some(6), None), vec![2, 3]);
        assert_eq!(s(Some(3), Some(6), None), vec![3]);
        assert_eq!(s(Some(4), Some(6), None), vec![]);
        assert_eq!(s(Some(5), Some(6), None), vec![]);
        assert_eq!(s(Some(4294967296), Some(17179869184), None), vec![]);
    }

    #[test]
    fn step() {
        const LEN: usize = 4;

        fn s(start: Option<isize>, end: Option<isize>, step: Option<isize>) -> Vec<usize> {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }.indices(LEN).collect()
        }

        assert_eq!(s(Some(0), Some(4), Some(1)), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(0), Some(4), Some(2)), vec![0, 2]);
        assert_eq!(s(Some(1), Some(4), Some(2)), vec![1, 3]);
        assert_eq!(s(Some(2), Some(4), Some(2)), vec![2]);
        assert_eq!(s(Some(3), Some(4), Some(2)), vec![3]);
        assert_eq!(s(Some(4), Some(4), Some(2)), vec![]);

        assert_eq!(s(Some(0), Some(4), Some(17179869184)), vec![0]);
        assert_eq!(s(Some(1), Some(4), Some(17179869184)), vec![1]);
        assert_eq!(s(Some(2), Some(4), Some(17179869184)), vec![2]);
        assert_eq!(s(Some(3), Some(4), Some(17179869184)), vec![3]);
        assert_eq!(s(Some(4), Some(4), Some(17179869184)), vec![]);
    }

    #[test]
    fn zero_step() {
        const LEN: usize = 4;

        fn s(start: Option<isize>, end: Option<isize>, step: Option<isize>) -> Vec<usize> {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }.indices(LEN).collect()
        }

        assert_eq!(s(Some(3), None, Some(0)), vec![]);
    }

    #[test]
    fn negative_step() {
        const LEN: usize = 4;

        fn s(start: Option<isize>, end: Option<isize>, step: Option<isize>) -> Vec<usize> {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }.indices(LEN).collect()
        }

        assert_eq!(s(Some(3), None, Some(-1)), vec![3, 2, 1, 0]);
        assert_eq!(s(Some(3), Some(0), Some(-1)), vec![3, 2, 1]);
        assert_eq!(s(Some(3), Some(1), Some(-1)), vec![3, 2]);
        assert_eq!(s(Some(3), Some(2), Some(-1)), vec![3]);
        assert_eq!(s(Some(3), Some(3), Some(-1)), vec![]);

        assert_eq!(s(Some(3), None, Some(-2)), vec![3, 1]);
        assert_eq!(s(Some(3), Some(0), Some(-2)), vec![3, 1]);
        assert_eq!(s(Some(3), Some(1), Some(-2)), vec![3]);
        assert_eq!(s(Some(3), Some(2), Some(-2)), vec![3]);
        assert_eq!(s(Some(3), Some(3), Some(-2)), vec![]);

        assert_eq!(s(Some(3), None, Some(-17179869184)), vec![3]);
        assert_eq!(s(Some(3), Some(0), Some(-17179869184)), vec![3]);
        assert_eq!(s(Some(3), Some(1), Some(-17179869184)), vec![3]);
        assert_eq!(s(Some(3), Some(2), Some(-17179869184)), vec![3]);
        assert_eq!(s(Some(3), Some(3), Some(-17179869184)), vec![]);

        assert_eq!(s(Some(17179869184), None, Some(-1)), vec![3, 2, 1, 0]);
        assert_eq!(s(Some(5), None, Some(-1)), vec![3, 2, 1, 0]);
        assert_eq!(s(Some(4), None, Some(-1)), vec![3, 2, 1, 0]);
    }

    #[test]
    fn negative_step_negative_start() {
        const LEN: usize = 4;

        fn s(start: Option<isize>, end: Option<isize>, step: Option<isize>) -> Vec<usize> {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }.indices(LEN).collect()
        }

        assert_eq!(s(Some(-1), None, Some(-1)), vec![3, 2, 1, 0]);
        assert_eq!(s(Some(-2), None, Some(-1)), vec![2, 1, 0]);
        assert_eq!(s(Some(-3), None, Some(-1)), vec![1, 0]);
        assert_eq!(s(Some(-4), None, Some(-1)), vec![0]);
        assert_eq!(s(Some(-5), None, Some(-1)), vec![]);
        assert_eq!(s(Some(-17179869184), None, Some(-1)), vec![]);
    }

    #[test]
    fn negative_step_negative_end() {
        const LEN: usize = 4;

        fn s(start: Option<isize>, end: Option<isize>, step: Option<isize>) -> Vec<usize> {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }.indices(LEN).collect()
        }

        assert_eq!(s(Some(3), Some(-5), Some(-1)), vec![3, 2, 1, 0]);
        assert_eq!(s(Some(3), Some(-4), Some(-1)), vec![3, 2, 1]);
        assert_eq!(s(Some(3), Some(-3), Some(-1)), vec![3, 2]);
        assert_eq!(s(Some(3), Some(-2), Some(-1)), vec![3]);
        assert_eq!(s(Some(3), Some(-1), Some(-1)), vec![]);

        assert_eq!(s(Some(3), Some(-5), Some(-2)), vec![3, 1]);
        assert_eq!(s(Some(3), Some(-4), Some(-2)), vec![3, 1]);
        assert_eq!(s(Some(3), Some(-3), Some(-2)), vec![3]);
        assert_eq!(s(Some(3), Some(-2), Some(-2)), vec![3]);
        assert_eq!(s(Some(3), Some(-1), Some(-2)), vec![]);

        assert_eq!(s(Some(3), Some(-5), Some(-17179869184)), vec![3]);
        assert_eq!(s(Some(3), Some(-4), Some(-17179869184)), vec![3]);
        assert_eq!(s(Some(3), Some(-3), Some(-17179869184)), vec![3]);
        assert_eq!(s(Some(3), Some(-2), Some(-17179869184)), vec![3]);
        assert_eq!(s(Some(3), Some(-1), Some(-17179869184)), vec![]);
    }

    #[test]
    fn empty_array() {
        const LEN: usize = 0;

        fn s(start: Option<isize>, end: Option<isize>, step: Option<isize>) -> Vec<usize> {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }.indices(LEN).collect()
        }

        assert_eq!(s(None, None, None), vec![]);
        assert_eq!(s(None, None, Some(-1)), vec![]);
    }

    #[test]
    fn display() {
        fn s(start: Option<isize>, end: Option<isize>, step: Option<isize>) -> Slice {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }
        }

        assert_eq!(s(None, None, None).to_string(), "[::]");
        assert_eq!(s(Some(0), None, None).to_string(), "[0::]");
        assert_eq!(s(Some(1), None, None).to_string(), "[1::]");
        assert_eq!(s(Some(-1), None, None).to_string(), "[-1::]");
        assert_eq!(s(None, Some(0), None).to_string(), "[:0:]");
        assert_eq!(s(None, Some(1), None).to_string(), "[:1:]");
        assert_eq!(s(None, Some(-1), None).to_string(), "[:-1:]");
        assert_eq!(s(None, None, Some(1)).to_string(), "[::1]");
        assert_eq!(s(None, None, Some(-1)).to_string(), "[::-1]");
    }
}
