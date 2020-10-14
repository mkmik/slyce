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
use std::ops::Bound;

/// A slice has an optional start, an optional end, and an optional step.
#[derive(Debug, Clone)]
pub struct Slice {
    pub start: Index,
    pub end: Index,
    pub step: Option<isize>,
}

/// A position inside an array.
///
/// Tail indices are represented with a distinct enumeration variant so that the full index
/// numeric range (usize) can be utilized without numeric overflows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Index {
    /// Position in the array relative to the start of the array (i.e. absolute position).
    Head(usize),
    /// Position in the array relative to the end of the array.
    Tail(usize),
    /// Either the first or the last element of the array, depending on the sign of `step`.
    Default,
}

use Index::*;

impl Slice {
    /// Returns an iterator that yields the elements that match the slice expression.
    pub fn apply<'a, T>(&self, arr: &'a [T]) -> impl Iterator<Item = &'a T> + 'a {
        self.indices(arr.len()).map(move |i| &arr[i])
    }

    /// Returns an iterator that yields the indices that match the slice expression.
    fn indices(&self, len: usize) -> impl Iterator<Item = usize> {
        let step = self.step.unwrap_or(1);
        let start = self
            .start
            .abs(len)
            .unwrap_or(if step >= 0 { 0 } else { len });
        let end = self.end.abs(len);
        let end = if step >= 0 {
            Bound::Excluded(end.unwrap_or(len))
        } else {
            // if iterating backwards, the only way to actually reach the first element of the
            // array is to use an "included" bound, which the user can only select by setting
            // end to Item::Default (which arrives here as a None).
            end.map_or(Bound::Included(0), Bound::Excluded)
        };
        RangeIterator::new((len - 1).min(start), end, step)
    }
}

impl Index {
    /// absolute index. negative indices are added to len.
    fn abs(&self, len: usize) -> Option<usize> {
        match self {
            &Head(n) => Some(len.min(n)),
            &Tail(n) => Some(len.saturating_sub(n)),
            Default => None,
        }
    }
}

/// An iterator that counts from an initial number up to a (included or excluded) final limit.
/// The direction and stride of the iteration can be controlled by the step parameter.
/// A zero step produces an empty iteration.
#[derive(Debug)]
struct RangeIterator {
    end: Bound<usize>,
    step: isize,
    pos: usize,
    done: bool,
    dir: Direction,
}

impl RangeIterator {
    fn new(start: usize, end: Bound<usize>, step: isize) -> Self {
        RangeIterator {
            pos: start,
            end,
            step,
            done: false,
            dir: if step >= 0 {
                Direction::Forwards
            } else {
                Direction::Backwards
            },
        }
    }
}

// Iteration direction, knows how to compare the limit index.
#[derive(Debug)]
enum Direction {
    Forwards,
    Backwards,
}

use Direction::*;

impl Direction {
    fn is_in_range(&self, pos: usize, end: usize) -> bool {
        match self {
            Forwards => pos < end,
            Backwards => pos > end,
        }
    }
}

impl Iterator for RangeIterator {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.step == 0 || self.done {
            return None;
        };

        let pos = self.pos;
        let new_pos = add_delta(self.pos, self.step);
        self.pos = new_pos.unwrap_or_default();
        self.done = new_pos.is_none();

        // the only way to stop iteration once we hit the "included" bound 0 is to
        // keep track of the fact and exit in the next iteration. This is because
        // we use unsigned indices and thus we cannot go lower than 0.
        if let Bound::Included(end) = self.end {
            if pos == end {
                self.done = true
            };
        };

        if match self.end {
            Bound::Excluded(end) => self.dir.is_in_range(pos, end),
            Bound::Included(end) => self.dir.is_in_range(pos, end) || self.done,
            Bound::Unbounded => true,
        } {
            Some(pos)
        } else {
            None
        }
    }
}

/// Add an unsigned integer to an unsigned base.
///
/// Uses saturated arithmetic since the array bounds cannot be
/// bigger than the usize range.
///
/// Returns None on underflow.
fn add_delta(n: usize, delta: isize) -> Option<usize> {
    if delta >= 0 {
        Some(n.saturating_add(delta as usize))
    } else {
        let r = n.wrapping_add(delta as usize);
        // return any underflow.
        if r > n {
            None
        } else {
            Some(r)
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
    fn demo() {
        const LEN: usize = 5;

        fn s(start: Option<isize>, end: Option<isize>, step: Option<isize>) -> Vec<usize> {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }.indices(LEN).collect()
        }

        assert_eq!(s(None, None, None), vec![0, 1, 2, 3, 4]);
        assert_eq!(s(Some(0), Some(5), None), vec![0, 1, 2, 3, 4]);
        assert_eq!(s(Some(0), Some(4), None), vec![0, 1, 2, 3]);
        assert_eq!(s(Some(1), None, None), vec![1, 2, 3, 4]);
        assert_eq!(s(None, Some(4), None), vec![0, 1, 2, 3]);
        assert_eq!(s(None, Some(-1), None), vec![0, 1, 2, 3]);
        assert_eq!(s(None, Some(-2), None), vec![0, 1, 2]);
        assert_eq!(s(Some(-2), Some(-1), None), vec![3]);
        assert_eq!(s(Some(-1), None, None), vec![4]);

        assert_eq!(s(None, None, Some(2)), vec![0, 2, 4]);

        assert_eq!(s(Some(4), Some(0), Some(-1)), vec![4, 3, 2, 1]);
        assert_eq!(s(Some(4), None, Some(-1)), vec![4, 3, 2, 1, 0]);
        assert_eq!(s(Some(4), None, Some(0)), vec![]);

        assert_eq!(s(Some(isize::MIN + 1), None, None), vec![0, 1, 2, 3, 4]);
        assert_eq!(s(None, Some(isize::MAX), None), vec![0, 1, 2, 3, 4]);
        assert_eq!(s(None, None, Some(100)), vec![0]);
        assert_eq!(s(None, None, Some(isize::MAX)), vec![0]);

        assert_eq!(s(Some(0), Some(6), Some(1)), vec![0, 1, 2, 3, 4]);
        assert_eq!(s(Some(6), Some(0), Some(-1)), vec![4, 3, 2, 1]);
        assert_eq!(s(None, Some(0), Some(-1)), vec![4, 3, 2, 1]);
    }

    #[test]
    fn even() {
        const LEN: usize = 4;

        fn s(start: Option<isize>, end: Option<isize>, step: Option<isize>) -> Vec<usize> {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }.indices(LEN).collect()
        }

        assert_eq!(s(None, None, Some(-2)), vec![3, 1]);
    }
}
