//! Slyce implements a python-like slicer for rust. It selects zero or more
//! elements from an input array and returns these elements, in the order
//! selected, as an output array.
//!
//! A slice selects array elements between an inclusive `start` index and an
//! exclusive `end` index, incrementing by a `step` value.
//!
//! Indices, of type `Index`, can be relative to the start of the array (Head), with Head(0)
//! denoting the first element of a non-empty array, or relative to the end
//! of the array (Tail), with Tail(1) denoting the last element of a non-empty
//! array.
//!
//! The default value for `step` is `1`.  The default values for `start`
//! and `end` depend on the sign of `step`, as follows (where `len` is the length
//! of the input array):
//! <table class="center" style="width:400px;margin-left:100px">
//!   <thead>
//!     <tr>
//!       <th class="text-left" rowspan="1" colspan="1" style="border:1px solid black">Condition</th>
//!       <th class="text-left" rowspan="1" colspan="1" style="border:1px solid black">start</th>
//!       <th class="text-left" rowspan="1" colspan="1" style="border:1px solid black">end</th>
//!     </tr>
//!   </thead>
//!   <tbody>
//!     <tr>
//!       <td class="text-left" rowspan="1" colspan="1" style="border:1px solid black">step &gt;= 0</td>
//!       <td class="text-left" rowspan="1" colspan="1" style="border:1px solid black">Head(0)</td>
//!       <td class="text-left" rowspan="1" colspan="1" style="border:1px solid black">Head(len)</td>
//!     </tr>
//!     <tr>
//!       <td class="text-left" rowspan="1" colspan="1" style="border:1px solid black">step &lt; 0</td>
//!       <td class="text-left" rowspan="1" colspan="1" style="border:1px solid black">Tail(1)</td>
//!       <td class="text-left" rowspan="1" colspan="1" style="border:1px solid black">Tail(len + 1)</td>
//!     </tr>
//!   </tbody>
//!   <caption>Default array slice start and end values</caption>
//! </table>
//!
//! If `start` or `end` is `Head(n)` for `n >= len` or `Tail(m)` for `m >= len + 1`, then it is
//! replaced by the default value for `start` or `end`, respectively.
//!
//! If `step` is zero, the resultant array is empty.
//!
//! If `step` is non-zero, iteration begins at the array index of `start` incrementing by `step` while
//! the array index of `end` has not been reached, with `end` itself excluded from the iteration. At
//! each iteration step, if the iteration value is inside the bounds of the array, the corresponding
//! element is selected and if the iteration value is outside the bounds of the array no element is
//! selected.
//!
//! This crate provides a few implementations of `From<T> for Index` for common types,
//! so you can pass numbers and options instead of Index (just call `.into()`).
//! An integer index value is equivalent to specifying a position
//! relative to the start or end of the array, depending on whether the value is
//! non-negative or negative, respectively. For example, `0` is
//! equivalent to `Head(0)` and `-1` is equivalent to `Tail(1)`.
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

use std::borrow::Borrow;
use std::default::Default;
use std::fmt;
use std::ops::RangeInclusive;

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
    /// Head(0) is the position of the first element of a non-empty array.
    Head(usize),
    /// Position in the array relative to the end of the array. Tail(1) is the position of the
    /// last element of a non-empty array.
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

impl Index {
    /// to_bound transforms an index slice parameter into an array bound.
    /// An array bound can be -1 in order to represent the exclusive lower
    /// bound 0.
    ///
    /// This impl won't work if size_of<usize> >= 16.
    /// 64 bits of address space should be enough for everybody.
    fn to_bound(&self, len: i128, r: &RangeInclusive<i128>) -> Option<i128> {
        match self {
            &Head(n) => Some(n as i128),
            &Tail(n) => Some(len - (n as i128)),
            Default => None,
        }
        .map(|n| clamp(n, r))
    }
}

fn clamp<T, R>(n: T, r: R) -> T
where
    T: Ord + Copy,
    R: Borrow<RangeInclusive<T>>,
{
    let r = r.borrow();
    let (start, end) = (*r.start(), *r.end());
    n.max(start).min(end)
}

impl Slice {
    /// Returns an iterator that yields the elements that match the slice expression.
    pub fn apply<'a, T>(&self, arr: &'a [T]) -> impl Iterator<Item = &'a T> + 'a {
        self.indices(arr.len()).map(move |i| &arr[i])
    }

    /// Returns an iterator that yields the indices that match the slice expression.
    fn indices(&self, ulen: usize) -> impl Iterator<Item = usize> {
        let len = ulen as i128;
        let step = self.step.unwrap_or(1);

        let (def_start, def_end) = if step >= 0 { (0, len) } else { (len - 1, -1) };

        let bounds = if step >= 0 {
            def_start..=def_end
        } else {
            def_end..=def_start
        };

        Iter {
            i: self.start.to_bound(len, &bounds).unwrap_or(def_start),
            end: self.end.to_bound(len, &bounds).unwrap_or(def_end),
            step: step as i128,
        }
    }
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

    fn slicer(len: usize) -> impl Fn(Option<isize>, Option<isize>, Option<isize>) -> Vec<usize> {
        move |start: Option<isize>, end: Option<isize>, step: Option<isize>| {
            let (start, end) = (start.into(), end.into());
            Slice { start, end, step }.indices(len).collect()
        }
    }

    #[test]
    fn positive() {
        let s = slicer(4);

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
        let s = slicer(4);

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
        let s = slicer(4);

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
        let s = slicer(4);

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
        let s = slicer(4);

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
        let s = slicer(4);

        assert_eq!(s(Some(3), None, Some(0)), vec![]);
    }

    #[test]
    fn negative_step() {
        let s = slicer(4);

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
        let s = slicer(4);

        assert_eq!(s(Some(-1), None, Some(-1)), vec![3, 2, 1, 0]);
        assert_eq!(s(Some(-2), None, Some(-1)), vec![2, 1, 0]);
        assert_eq!(s(Some(-3), None, Some(-1)), vec![1, 0]);
        assert_eq!(s(Some(-4), None, Some(-1)), vec![0]);
        assert_eq!(s(Some(-5), None, Some(-1)), vec![]);
        assert_eq!(s(Some(-17179869184), None, Some(-1)), vec![]);
    }

    #[test]
    fn negative_step_negative_end() {
        let s = slicer(4);

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
        let s = slicer(0);

        assert_eq!(s(None, None, None), vec![]);
        assert_eq!(s(None, None, Some(-1)), vec![]);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(0, 1..=3), 1);
        assert_eq!(clamp(1, 1..=3), 1);
        assert_eq!(clamp(2, 1..=3), 2);
        assert_eq!(clamp(3, 1..=3), 3);
        assert_eq!(clamp(4, 1..=3), 3);
        assert_eq!(clamp(5, 1..=3), 3);
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
