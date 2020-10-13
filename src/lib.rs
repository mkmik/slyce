//! Slyce implements a python-like slicer for rust.
//!
//! TODO

#![allow(unused_imports)]

#[derive(Debug)]
pub struct Slice {
    pub start: Index,
    pub end: Index,
    pub step: Index,
}

#[derive(Debug)]
pub enum Index {
    Positive(usize),
    Negative(usize),
    Default,
}

use Index::*;

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
        let i0 = Some(-123 as isize);
        let s = Slice {
            start: i0.into(),
            end: Default,
            step: Default,
        };
        println!("xxx: {:?}", s);
        assert!(false);
    }
}
