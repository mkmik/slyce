![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# slyce

Slyce implements a python-like slicer for rust.

TODO
## Example
```rust
use slyce::{Slice, Index};
let s = slyce::Slice{start: Index::Negative(3), end: Index::Default, step: None};
let v = vec![10,20,30,40,50];
let it = s.apply(&v);
assert_eq!(format!("{:?}", it.collect::<Vec<_>>()), "[30, 40, 50]");
```

Current version: 0.1.0

License: BSD-2-Clause
