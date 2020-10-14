//! Slyce implements a python-like slicer for rust.
//!
//! Slice indices are represented with an enum that wraps the full `usize` range, but also
//! captures the possibility of a "negative" or "backward" index.
//! This crate provides a few implementations of `From<T> for Index` for common types,
//! so you can pass numbers and options instead of Index.
//!
//! # Example
//! ```
//! use slyce::{Slice, Index};
//! let v = vec![10,20,30,40,50];
//!
//! let start: isize = -3;
//! let s = slyce::Slice{start: start.into(), end: Index::Default, step: None};
//! let it = s.apply(&v);
//! assert_eq!(format!("{:?}", it.collect::<Vec<_>>()), "[30, 40, 50]");
//!
//! let s = slyce::Slice{start: Index::Tail(3), end: Index::Default, step: Some(-1)};
//! let it = s.apply(&v);
//! assert_eq!(format!("{:?}", it.collect::<Vec<_>>()), "[30, 20, 10]");
//!
//! let s = slyce::Slice{start: Index::Head(4), end: Index::Head(0), step: Some(-1)};
//! let it = s.apply(&v);
//! assert_eq!(format!("{:?}", it.collect::<Vec<_>>()), "[50, 40, 30, 20]");
//!
//! let s = slyce::Slice{start: Index::Default, end: Index::Head(0), step: Some(-1)};
//! let it = s.apply(&v);
//! assert_eq!(format!("{:?}", it.collect::<Vec<_>>()), "[50, 40, 30, 20]");
//! ```

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
#[derive(Debug, Clone)]
pub enum Index {
    Head(usize),
    Tail(usize),
    Default,
}

use Index::*;

impl Slice {
    /// Returns an iterator that yields the elements that match the slice expression.
    pub fn apply<'a, T>(self, arr: &'a [T]) -> impl Iterator<Item = &'a T> + 'a {
        self.indices(arr.len()).map(move |i| &arr[i])
    }

    /// Returns an iterator that yields the indices that match the slice expression.
    fn indices(self, len: usize) -> impl Iterator<Item = usize> {
        let step = self.step.unwrap_or(1);
        let start = (len - 1).min(
            self.start
                .abs(len)
                .unwrap_or(if step >= 0 { 0 } else { len }),
        );
        let end = self.end.abs(len);
        let end = if step >= 0 {
            Bound::Excluded(end.unwrap_or(len))
        } else {
            end.map_or(Bound::Included(0), Bound::Excluded)
        };
        SliceIterator {
            end: end,
            step: step,
            cur: start,
            done: false,
        }
        .fuse()
        .take(10)
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

/// A slice iterator returns index positions for a given range.
struct SliceIterator {
    end: Bound<usize>,
    step: isize,
    cur: usize,
    done: bool,
}

impl Iterator for SliceIterator {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.step == 0 || self.done {
            return None;
        };

        let cur = self.cur;
        self.cur = add_delta(self.cur, self.step);

        if let Bound::Included(end) = self.end {
            if cur == end {
                self.done = true
            };
        };

        if if self.step > 0 {
            match self.end {
                Bound::Excluded(end) => cur < end,
                Bound::Included(end) => cur <= end,
                Bound::Unbounded => true,
            }
        } else {
            match self.end {
                Bound::Excluded(end) => cur > end,
                Bound::Included(end) => cur >= end,
                Bound::Unbounded => true,
            }
        } {
            Some(cur)
        } else {
            None
        }
    }
}

/// Add an unsigned integer to an unsigned base.
///
/// Uses saturated arithmetic since the array bounds cannot be
/// bigger than the usize range.
fn add_delta(n: usize, delta: isize) -> usize {
    if delta >= 0 {
        n.saturating_add(delta as usize)
    } else {
        let r = n.wrapping_add(delta as usize);
        // manually saturate to 0 in case of underflow.
        if r > n {
            0
        } else {
            r
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
}
