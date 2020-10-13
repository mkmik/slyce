//! Slyce implements a python-like slicer for rust.
//!
//! TODO
//! # Example
//! ```
//! use slyce::{Slice, Index};
//! slyce::Slice{start: 12.into(), end: Index::Default, step: None};
//! ```

#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::ops::Bound;

#[derive(Debug, Clone)]
pub struct Slice {
    pub start: Index,
    pub end: Index,
    pub step: Option<isize>,
}

#[derive(Debug, Clone)]
pub enum Index {
    Positive(usize),
    Negative(usize),
    Default,
}

use Index::*;

impl Slice {
    pub fn apply<'a, T>(self, arr: &'a [T]) -> impl Iterator<Item = &'a T> + 'a {
        self.indices(arr.len()).map(move |i| &arr[i])
    }

    fn indices(self, len: usize) -> SliceIterator {
        println!("Indices, relative: {:?}", self.start.relative(len));
        let start = self.start.relative(len).unwrap_or(0);
        SliceIterator {
            start: Bound::Included(start),
            end: Bound::Excluded(self.end.relative(len).unwrap_or(len)),
            step: self.step.unwrap_or(1),
            cur: start,
        }
    }
}

impl Index {
    fn relative(&self, len: usize) -> Option<usize> {
        match self {
            &Positive(n) => Some(len.min(n)),
            &Negative(n) => Some(len.saturating_sub(n)),
            Default => None,
        }
    }
}

struct SliceIterator {
    start: Bound<usize>,
    end: Bound<usize>,
    step: isize,
    cur: usize,
}

impl Iterator for SliceIterator {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let cur = self.cur;
        self.cur = add_delta(self.cur, self.step);
        if match self.end {
            Bound::Excluded(end) => cur < end,
            Bound::Included(end) => cur <= end,
            Bound::Unbounded => true,
        } {
            Some(cur)
        } else {
            None
        }
    }
}

fn add_delta(n: usize, delta: isize) -> usize {
    n.wrapping_add(delta as usize)
}

impl From<usize> for Index {
    fn from(i: usize) -> Self {
        Positive(i)
    }
}

impl From<isize> for Index {
    fn from(i: isize) -> Self {
        if i < 0 {
            Negative(-i as usize)
        } else {
            Positive(i as usize)
        }
    }
}

impl From<i32> for Index {
    fn from(i: i32) -> Self {
        if i < 0 {
            Negative(-i as usize)
        } else {
            Positive(i as usize)
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
        assert_eq!(s(Some(1), None, None), vec![1, 2, 3, 4]);
        assert_eq!(s(None, Some(4), None), vec![0, 1, 2, 3]);
        assert_eq!(s(None, Some(-1), None), vec![0, 1, 2, 3]);
        assert_eq!(s(None, Some(-2), None), vec![0, 1, 2]);
        assert_eq!(s(Some(-2), Some(-1), None), vec![3]);
        assert_eq!(s(Some(-1), None, None), vec![4]);

        assert_eq!(s(None, None, Some(2)), vec![0, 2, 4]);
    }
}
