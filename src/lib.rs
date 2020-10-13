#![allow(unused_imports)]

pub struct Slice {
    pub start: Index,
    pub end: Index,
    pub step: Index,
}

pub enum Index {
    Positive(usize),
    Negative(usize),
    Default,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn demo() {}
}
