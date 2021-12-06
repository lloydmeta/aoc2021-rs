use std::result::Result as StdResult;

use anyhow::Result;
use combine::parser::char::*;
use combine::*;

use crate::common::usize_parser;

const INPUT: &str = include_str!("../data/day_06_input");
const BORN_WITH_TIMER: usize = 8;
const TIMER_AFTER_GIVING_BIRTH: usize = 6;

pub fn run() -> Result<()> {
    println!("*** Day 6: Lanternfish ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let part_1_diagram = part_1_play(&input, 80);
    println!("Solution 1: {:?}\n", part_1_diagram.count_fishes());
    let part_2_diagram = part_1_play(&input, 256);
    println!("Solution 2: {:?}\n", part_2_diagram.count_fishes());
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Input(Vec<usize>);

#[derive(Debug, Eq, PartialEq)]
struct State(Vec<usize>);

impl State {
    fn count_fishes(&self) -> usize {
        self.0.iter().sum()
    }
}

fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
    let mut parser = sep_by1(usize_parser(), char(',')).map(Input);
    let (r, _) = parser.easy_parse(s)?;
    Ok(r)
}

fn part_1_play(i: &Input, days_to_play: usize) -> State {
    let starting_state = {
        i.0.iter()
            .fold(vec![0; BORN_WITH_TIMER + 1], |mut acc, next| {
                acc[*next] += 1;
                acc
            })
    };
    let finished_state = (0..days_to_play).fold(starting_state, |mut acc, _day| {
        let new_born_count = acc[0];
        acc.copy_within(1.., 0);
        acc[BORN_WITH_TIMER] = new_born_count;
        acc[TIMER_AFTER_GIVING_BIRTH] += new_born_count;
        acc
    });
    State(finished_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &str = "3,4,3,1,2";

    #[test]
    fn input_parse_test() {
        let r = parse(TEST_INPUT).unwrap();
        let expected = Input(vec![3, 4, 3, 1, 2]);
        assert_eq!(expected, r);
    }

    #[test]
    fn day_1_play_test() {
        let r = parse(TEST_INPUT).unwrap();
        let s = part_1_play(&r, 18);
        assert_eq!(5934, part_1_play(&r, 80).count_fishes())
    }
}
