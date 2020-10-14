![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# slyce

Slyce implements a python-like slicer for rust.

TODO
## Example
```rust
use slyce::{Slice, Index};
let v = vec![10,20,30,40,50];

let s = slyce::Slice{start: Index::Negative(3), end: Index::Default, step: None};
let it = s.apply(&v);
assert_eq!(format!("{:?}", it.collect::<Vec<_>>()), "[30, 40, 50]");

let s = slyce::Slice{start: Index::Negative(3), end: Index::Default, step: Some(-1)};
let it = s.apply(&v);
assert_eq!(format!("{:?}", it.collect::<Vec<_>>()), "[30, 20, 10]");

let s = slyce::Slice{start: Index::Positive(4), end: Index::Positive(0), step: Some(-1)};
let it = s.apply(&v);
assert_eq!(format!("{:?}", it.collect::<Vec<_>>()), "[50, 40, 30, 20]");

let s = slyce::Slice{start: Index::Default, end: Index::Positive(0), step: Some(-1)};
let it = s.apply(&v);
assert_eq!(format!("{:?}", it.collect::<Vec<_>>()), "[50, 40, 30, 20]");
```

Current version: 0.1.0

License: BSD-2-Clause
