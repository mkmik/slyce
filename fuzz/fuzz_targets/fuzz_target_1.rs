#![no_main]
use libfuzzer_sys::{arbitrary, fuzz_target};
use slyce::{Index, Slice};
use std::process::Command;

#[derive(arbitrary::Arbitrary, Debug)]
struct Input {
    data: Vec<u8>,
    slice: Slice,
}

fuzz_target!(|input: Input| {
    // TODO: find a better way to avoid generating impossible input Tail(0)
    if input.slice.start == Index::Tail(0) || input.slice.end == Index::Tail(0) {
        return;
    }
    // python errors if step is zero, while slyce returns an empty slice. currently this is intentional.
    if input.slice.step == Some(0) {
        return;
    }

    let r: Vec<&u8> = input.slice.apply(&input.data).collect();

    let pyout = Command::new("python")
        .arg("-c")
        .arg(format!("print({:?}{})", input.data, input.slice))
        .output()
        .expect("failed to execute process");

    let mut py = std::str::from_utf8(&pyout.stdout).unwrap().to_string();
    let len = py.trim_end_matches(&['\r', '\n'][..]).len();
    py.truncate(len);

    assert_eq!(py, format!("{:?}", r));
});
