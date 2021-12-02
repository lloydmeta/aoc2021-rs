use std::num::ParseIntError;

use combine::parser::char::*;
use combine::*;

pub(crate) fn usize_parser<Input>() -> impl Parser<Input, Output = usize>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    <<Input as StreamOnce>::Error as combine::ParseError<
        char,
        <Input as StreamOnce>::Range,
        <Input as StreamOnce>::Position,
    >>::StreamError: From<ParseIntError>,
    <<Input as StreamOnce>::Error as combine::ParseError<
        char,
        <Input as StreamOnce>::Range,
        <Input as StreamOnce>::Position,
    >>::StreamError: From<ParseIntError>,
    <Input as combine::StreamOnce>::Error: combine::ParseError<
        char,
        <Input as combine::StreamOnce>::Range,
        <Input as combine::StreamOnce>::Position,
    >,
{
    many::<String, _, _>(digit()).and_then(|num| num.parse())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_to_usize_test() {
        let (r, _) = usize_parser().easy_parse("123").unwrap();
        assert_eq!(123, r);
    }
}
