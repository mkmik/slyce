#![allow(unused_imports)]
#![allow(unused_variables)]

use nom::{
    character::complete::{char, digit1},
    combinator::{map_res, opt},
    sequence::tuple,
    IResult,
};
use serde_json;
use slyce::{Index, Slice};
use std::env;
use std::io;

fn parse(input: &str) -> io::Result<Slice> {
    parse_slice(input)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("{}", e)))
        .map(|v| v.1)
}

fn parse_index(input: &str) -> IResult<&str, Option<isize>> {
    let (input, sign) = opt(char('-'))(input)?;
    let (input, digits) = opt(map_res(digit1, |s: &str| s.parse::<isize>()))(input)?;
    Ok((
        input,
        digits.map(|d| match sign {
            Some(_) => -d,
            None => d,
        }),
    ))
}

fn parse_slice(input: &str) -> IResult<&str, Slice> {
    let (input, _) = char('[')(input)?;
    let (input, start) = parse_index(input)?;
    let (input, _) = char(':')(input)?;
    let (input, end) = parse_index(input)?;
    let (input, _) = char(':')(input)?;
    let (input, step) = parse_index(input)?;
    let (input, _) = char(']')(input)?;

    Ok((
        input,
        Slice {
            start: start.into(),
            end: end.into(),
            step: step.into(),
        },
    ))
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: echo '[1,2,3]' | demo '[<slice expression>]'",
        ));
    }
    let expr = &args[1];
    let slice = parse(expr)?;

    let arr: Vec<i32> = serde_json::from_reader(io::stdin())?;
    println!("{:?}", slice.apply(&arr).collect::<Vec<_>>());

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse() -> io::Result<()> {
        let s = parse("[:-2:1]")?;

        assert_eq!(s.start, Index::Default);
        assert_eq!(s.end, Index::Tail(2));
        assert_eq!(s.step, Some(1));

        Ok(())
    }
}
