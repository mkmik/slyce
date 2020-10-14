![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# slyce

Slyce implements a python-like slicer for rust.

Indices can be addressed as absolute positions or relatively to the end of the array (Tail).
Out of bound indices are simply omitted.

Slice indices are represented with an enum that wraps the full `usize` range, but also
captures the possibility of a "negative" or "backward" index.
This crate provides a few implementations of `From<T> for Index` for common types,
so you can pass numbers and options instead of Index (just call `.into()`).

## Example
```rust
use slyce::{Slice, Index};
let v = vec![10,20,30,40,50];
let render = |s: Slice| format!("{:?}", s.apply(&v).collect::<Vec<_>>());

let start: isize = -3;
let s = slyce::Slice{start: start.into(), end: Index::Default, step: None};
assert_eq!(render(s), "[30, 40, 50]");

let s = slyce::Slice{start: Index::Tail(3), end: Index::Default, step: None};
assert_eq!(render(s), "[30, 40, 50]");

let end: Option<isize> = None;
let s = slyce::Slice{start: Index::Tail(3), end: end.into(), step: None};
assert_eq!(render(s), "[30, 40, 50]");

let s = slyce::Slice{start: Index::Tail(3), end: Index::Default, step: Some(-1)};
assert_eq!(render(s), "[30, 20, 10]");

let s = slyce::Slice{start: Index::Head(4), end: Index::Head(0), step: Some(-1)};
assert_eq!(render(s), "[50, 40, 30, 20]");

let s = slyce::Slice{start: Index::Default, end: Index::Head(0), step: Some(-1)};
assert_eq!(render(s), "[50, 40, 30, 20]");

let s = slyce::Slice{start: Index::Tail(1000), end: 2000.into(), step: None};
assert_eq!(render(s), "[10, 20, 30, 40, 50]");
```

Current version: 0.1.4

License: BSD-2-Clause
