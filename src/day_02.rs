use std::result::Result as StdResult;

use anyhow::Result;
use combine::easy;
use combine::parser::char::*;
use combine::*;

use Move::*;

use crate::common::usize_parser;

const INPUT: &str = include_str!("../data/day_02_input");

pub fn run() -> Result<()> {
    println!("*** Day 2: Dive! ***");
    println!("Input: {}", INPUT);
    let program = parse(INPUT)?;
    let position_1 = run_prog_1(&program);
    println!("Solution 1: {:?}\n", position_1.solution());

    let sub = run_prog_2(&program);
    println!("Solution 2: {:?}\n", sub.position.solution());
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Program(Vec<Move>);

#[derive(Debug, Eq, PartialEq)]
struct Position {
    x: isize,
    y: isize,
}

impl Position {
    fn solution(&self) -> isize {
        self.x * self.y
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Sub {
    position: Position,
    aim: isize,
}

#[derive(Debug, Eq, PartialEq)]
enum Move {
    Forward(usize),
    Down(usize),
    Up(usize),
}

fn parse(s: &str) -> StdResult<Program, easy::ParseError<&str>> {
    let single_instruction_parser = attempt(
        string("forward")
            .skip(space())
            .and(usize_parser::<_>())
            .map(|(_, num)| Move::Forward(num))
            .or(attempt(
                string("down")
                    .skip(space())
                    .and(usize_parser())
                    .map(|(_, num)| Move::Down(num)),
            ))
            .or(attempt(
                string("up")
                    .skip(space())
                    .and(usize_parser())
                    .map(|(_, num)| Move::Up(num)),
            )),
    );
    let mut parser = many(single_instruction_parser.skip(spaces())).map(Program);
    let (r, _) = parser.easy_parse(s)?;
    Ok(r)
}

fn run_prog_1(p: &Program) -> Position {
    p.0.iter()
        .fold(Position { x: 0, y: 0 }, |acc, next| match next {
            Forward(i) => Position {
                x: acc.x + *i as isize,
                ..acc
            },
            Down(i) => Position {
                y: acc.y + *i as isize,
                ..acc
            },
            Up(i) => Position {
                y: acc.y - *i as isize,
                ..acc
            },
        })
}

fn run_prog_2(p: &Program) -> Sub {
    p.0.iter().fold(
        Sub {
            position: Position { x: 0, y: 0 },
            aim: 0,
        },
        |acc, next| match next {
            Forward(i) => Sub {
                position: Position {
                    x: acc.position.x + *i as isize,
                    y: acc.position.y + acc.aim * *i as isize,
                },
                ..acc
            },
            Down(i) => Sub {
                aim: acc.aim + *i as isize,
                ..acc
            },
            Up(i) => Sub {
                aim: acc.aim - *i as isize,
                ..acc
            },
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        let input = "forward 5
down 5
forward 8
up 3
down 8
forward 2";
        let p = parse(input).unwrap();
        assert_eq!(
            vec![Forward(5), Down(5), Forward(8), Up(3), Down(8), Forward(2)],
            p.0
        );
    }

    #[test]
    fn run_prog_1_test() {
        let program = Program(vec![
            Forward(5),
            Down(5),
            Forward(8),
            Up(3),
            Down(8),
            Forward(2),
        ]);
        let r = run_prog_1(&program);
        assert_eq!(Position { x: 15, y: 10 }, r);
    }

    #[test]
    fn run_prog_2_test() {
        let program = Program(vec![
            Forward(5),
            Down(5),
            Forward(8),
            Up(3),
            Down(8),
            Forward(2),
        ]);
        let r = run_prog_2(&program);
        assert_eq!(Position { x: 15, y: 60 }, r.position);
    }
}
