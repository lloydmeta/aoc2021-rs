use std::result::Result as StdResult;

use anyhow::Result;
use combine::parser::char::*;
use combine::*;
use itertools::{FoldWhile, Itertools};

use Bracket::*;
use Char::*;

const INPUT: &str = include_str!("../data/day_10_input");

pub fn run() -> Result<()> {
    println!("*** Day 10: Syntax Scoring ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let sol_1 = sol_1(&input);
    println!("Solution 1: {:?}", sol_1);
    let sol_2 = sol_2(&input);
    println!("Solution 2: {:?}", sol_2);
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Input(Vec<Vec<Char>>);

#[derive(Debug, Eq, PartialEq)]
enum Char {
    Open(Bracket),
    Close(Bracket),
}

#[derive(Debug, Eq, PartialEq)]
enum Bracket {
    Square,
    Paren,
    Curly,
    Angle,
}

impl Bracket {
    fn points(&self) -> usize {
        match *self {
            Paren => 3,
            Square => 57,
            Curly => 1197,
            Angle => 25137,
        }
    }

    fn auto_complete_points(&self) -> usize {
        match *self {
            Paren => 1,
            Square => 2,
            Curly => 3,
            Angle => 4,
        }
    }
}

fn sol_1(i: &Input) -> usize {
    i.0.iter().fold(0, |acc, line| {
        if let Some(first_illegal) = find_first_illegal(line).1 {
            acc + first_illegal.points()
        } else {
            acc
        }
    })
}

fn sol_2(i: &Input) -> usize {
    let scores: Vec<_> =
        i.0.iter()
            .map(|line| {
                let (remaining_stack, maybe_illegal) = find_first_illegal(line);
                if maybe_illegal.is_none() {
                    // incomplete line
                    remaining_stack
                        .iter()
                        .rev()
                        .fold(0usize, |acc, next_bracket| {
                            acc * 5 + next_bracket.auto_complete_points()
                        })
                } else {
                    0
                }
            })
            .filter(|s| *s != 0)
            .sorted()
            .collect();
    scores[scores.len() / 2]
}

fn find_first_illegal(line: &[Char]) -> (Vec<&Bracket>, Option<&Bracket>) {
    let (stack, illegal_closer) = line
        .iter()
        .enumerate()
        .fold_while(
            (Vec::with_capacity(line.len()), None),
            |(mut stack, illegal_closer), (_idx, char)| {
                match char {
                    Open(open_bracket) => {
                        // do nothing, simply add to stack and move on
                        stack.push(open_bracket);
                        FoldWhile::Continue((stack, illegal_closer))
                    }
                    Close(close_bracket) => {
                        if let Some(last_open_bracket) = stack.pop() {
                            if last_open_bracket == close_bracket {
                                FoldWhile::Continue((stack, illegal_closer))
                            } else {
                                FoldWhile::Done((stack, Some(close_bracket)))
                            }
                        } else {
                            // nothing in the stack, just keep going
                            FoldWhile::Continue((stack, illegal_closer))
                        }
                    }
                }
            },
        )
        .into_inner();

    (stack, illegal_closer)
}

fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
    let line_parser = many1(char_parser());
    let mut parser = many1(line_parser.skip(spaces())).map(Input);
    let (r, _) = parser.easy_parse(s)?;
    Ok(r)
}

fn char_parser<Input>() -> impl Parser<Input, Output = Char>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    <Input as combine::StreamOnce>::Error: combine::ParseError<
        char,
        <Input as combine::StreamOnce>::Range,
        <Input as combine::StreamOnce>::Position,
    >,
{
    attempt(char('[').map(|_| Open(Square)))
        .or(attempt(char('(').map(|_| Open(Paren))))
        .or(attempt(char('{').map(|_| Open(Curly))))
        .or(attempt(char('<').map(|_| Open(Angle))))
        .or(attempt(char(']').map(|_| Close(Square))))
        .or(attempt(char(')').map(|_| Close(Paren))))
        .or(attempt(char('}').map(|_| Close(Curly))))
        .or(attempt(char('>').map(|_| Close(Angle))))
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &str = "[({(<(())[]>[[{[]{<()<>>
[(()[<>])]({[<{<<[]>>(
{([(<{}[<>[]}>{[]{[(<()>
(((({<>}<{<{<>}{[]{[]{}
[[<[([]))<([[{}[[()]]]
[{[{({}]{}}([{[{{{}}([]
{<[[]]>}<{[{[{[]{()[[[]
[<(<(<(<{}))><([]([]()
<{([([[(<>()){}]>(<<{{
<{([{{}}[<[[[<>{}]]]>[]]";

    #[test]
    fn sol_1_test() {
        let i = parse(TEST_INPUT).unwrap();
        let s = sol_1(&i);
        assert_eq!(26397, s);
    }

    #[test]
    fn sol_2_test() {
        let i = parse(TEST_INPUT).unwrap();
        let s = sol_2(&i);
        assert_eq!(288957, s);
    }

    #[test]
    fn parse_test() {
        let i = parse(TEST_INPUT).unwrap();
        assert_eq!(
            Input(vec![
                vec![
                    Open(Square),
                    Open(Paren),
                    Open(Curly),
                    Open(Paren),
                    Open(Angle),
                    Open(Paren),
                    Open(Paren),
                    Close(Paren),
                    Close(Paren),
                    Open(Square),
                    Close(Square),
                    Close(Angle),
                    Open(Square),
                    Open(Square),
                    Open(Curly),
                    Open(Square),
                    Close(Square),
                    Open(Curly),
                    Open(Angle),
                    Open(Paren),
                    Close(Paren),
                    Open(Angle),
                    Close(Angle),
                    Close(Angle)
                ],
                vec![
                    Open(Square),
                    Open(Paren),
                    Open(Paren),
                    Close(Paren),
                    Open(Square),
                    Open(Angle),
                    Close(Angle),
                    Close(Square),
                    Close(Paren),
                    Close(Square),
                    Open(Paren),
                    Open(Curly),
                    Open(Square),
                    Open(Angle),
                    Open(Curly),
                    Open(Angle),
                    Open(Angle),
                    Open(Square),
                    Close(Square),
                    Close(Angle),
                    Close(Angle),
                    Open(Paren)
                ],
                vec![
                    Open(Curly),
                    Open(Paren),
                    Open(Square),
                    Open(Paren),
                    Open(Angle),
                    Open(Curly),
                    Close(Curly),
                    Open(Square),
                    Open(Angle),
                    Close(Angle),
                    Open(Square),
                    Close(Square),
                    Close(Curly),
                    Close(Angle),
                    Open(Curly),
                    Open(Square),
                    Close(Square),
                    Open(Curly),
                    Open(Square),
                    Open(Paren),
                    Open(Angle),
                    Open(Paren),
                    Close(Paren),
                    Close(Angle)
                ],
                vec![
                    Open(Paren),
                    Open(Paren),
                    Open(Paren),
                    Open(Paren),
                    Open(Curly),
                    Open(Angle),
                    Close(Angle),
                    Close(Curly),
                    Open(Angle),
                    Open(Curly),
                    Open(Angle),
                    Open(Curly),
                    Open(Angle),
                    Close(Angle),
                    Close(Curly),
                    Open(Curly),
                    Open(Square),
                    Close(Square),
                    Open(Curly),
                    Open(Square),
                    Close(Square),
                    Open(Curly),
                    Close(Curly)
                ],
                vec![
                    Open(Square),
                    Open(Square),
                    Open(Angle),
                    Open(Square),
                    Open(Paren),
                    Open(Square),
                    Close(Square),
                    Close(Paren),
                    Close(Paren),
                    Open(Angle),
                    Open(Paren),
                    Open(Square),
                    Open(Square),
                    Open(Curly),
                    Close(Curly),
                    Open(Square),
                    Open(Square),
                    Open(Paren),
                    Close(Paren),
                    Close(Square),
                    Close(Square),
                    Close(Square)
                ],
                vec![
                    Open(Square),
                    Open(Curly),
                    Open(Square),
                    Open(Curly),
                    Open(Paren),
                    Open(Curly),
                    Close(Curly),
                    Close(Square),
                    Open(Curly),
                    Close(Curly),
                    Close(Curly),
                    Open(Paren),
                    Open(Square),
                    Open(Curly),
                    Open(Square),
                    Open(Curly),
                    Open(Curly),
                    Open(Curly),
                    Close(Curly),
                    Close(Curly),
                    Open(Paren),
                    Open(Square),
                    Close(Square)
                ],
                vec![
                    Open(Curly),
                    Open(Angle),
                    Open(Square),
                    Open(Square),
                    Close(Square),
                    Close(Square),
                    Close(Angle),
                    Close(Curly),
                    Open(Angle),
                    Open(Curly),
                    Open(Square),
                    Open(Curly),
                    Open(Square),
                    Open(Curly),
                    Open(Square),
                    Close(Square),
                    Open(Curly),
                    Open(Paren),
                    Close(Paren),
                    Open(Square),
                    Open(Square),
                    Open(Square),
                    Close(Square)
                ],
                vec![
                    Open(Square),
                    Open(Angle),
                    Open(Paren),
                    Open(Angle),
                    Open(Paren),
                    Open(Angle),
                    Open(Paren),
                    Open(Angle),
                    Open(Curly),
                    Close(Curly),
                    Close(Paren),
                    Close(Paren),
                    Close(Angle),
                    Open(Angle),
                    Open(Paren),
                    Open(Square),
                    Close(Square),
                    Open(Paren),
                    Open(Square),
                    Close(Square),
                    Open(Paren),
                    Close(Paren)
                ],
                vec![
                    Open(Angle),
                    Open(Curly),
                    Open(Paren),
                    Open(Square),
                    Open(Paren),
                    Open(Square),
                    Open(Square),
                    Open(Paren),
                    Open(Angle),
                    Close(Angle),
                    Open(Paren),
                    Close(Paren),
                    Close(Paren),
                    Open(Curly),
                    Close(Curly),
                    Close(Square),
                    Close(Angle),
                    Open(Paren),
                    Open(Angle),
                    Open(Angle),
                    Open(Curly),
                    Open(Curly)
                ],
                vec![
                    Open(Angle),
                    Open(Curly),
                    Open(Paren),
                    Open(Square),
                    Open(Curly),
                    Open(Curly),
                    Close(Curly),
                    Close(Curly),
                    Open(Square),
                    Open(Angle),
                    Open(Square),
                    Open(Square),
                    Open(Square),
                    Open(Angle),
                    Close(Angle),
                    Open(Curly),
                    Close(Curly),
                    Close(Square),
                    Close(Square),
                    Close(Square),
                    Close(Angle),
                    Open(Square),
                    Close(Square),
                    Close(Square)
                ]
            ]),
            i
        );
    }
}
