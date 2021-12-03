use std::result::Result as StdResult;

use anyhow::Result;
use combine::easy;
use combine::parser::char::*;
use combine::stream::easy::{Error, Info};
use combine::*;
use itertools::{FoldWhile, Itertools};

const INPUT: &str = include_str!("../data/day_03_input");

pub fn run() -> Result<()> {
    println!("*** Day 3: Binary Diagnostic ***");
    println!("Input: {}", INPUT);
    let input = Input::parse(INPUT)?;
    let p_c = power_consumption(&input);
    println!("Solution 1: {:?}\n", p_c);
    let life_support_rating = life_support_rating(&input);
    println!("Solution 2: {:?}\n", life_support_rating);
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct PowerConsumption(u64);

#[derive(Debug, Eq, PartialEq)]
struct OxygenGeneratorRating(u64);

#[derive(Debug, Eq, PartialEq)]
struct CO2ScrubberRating(u64);

#[derive(Debug, Eq, PartialEq)]
struct LifeSupportRating(u64);

#[derive(Debug, Eq, PartialEq)]
struct Input {
    vecs: Vec<Vec<bool>>,
    bits: usize,
}

impl Input {
    fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
        let single = many1(char('0').map(|_| false).or(char('1').map(|_| true)));
        let mut parser = many1(single.skip(spaces())).and_then(|vecs: Vec<Vec<bool>>| {
            if let Some(first) = vecs.get(0) {
                let first_len = first.len();
                if vecs.iter().all(|v| v.len() == first_len) {
                    Ok(Input {
                        vecs,
                        bits: first_len,
                    })
                } else {
                    Err(Error::Unexpected(Info::Static("Non-rectangular input")))
                }
            } else {
                Err(Error::Unexpected(Info::Static("No input")))
            }
        });
        let (r, _) = parser.easy_parse(s)?;
        Ok(r)
    }
}

fn power_consumption(i: &Input) -> PowerConsumption {
    // use this as a threshold
    let vecs_length = i.vecs.len() as u64;
    let more_ones_than_zeroes_threshold = vecs_length / 2;

    let most_common_bits = (0..i.bits).into_iter().fold(0u64, |acc, idx| {
        let sum_of_bits_at_idx = i.vecs.iter().fold(0, |acc, vec| {
            // direct vec idx is fine here because Input is parsed and all rows have the same length
            if vec[idx] {
                acc + 1
            } else {
                acc
            }
        });
        let more_ones_than_zero_at_bit = sum_of_bits_at_idx > more_ones_than_zeroes_threshold;
        if more_ones_than_zero_at_bit {
            let shift = (i.bits - 1 - idx) as u64;
            let mask = 1 << shift;
            acc | mask
        } else {
            acc
        }
    });
    let least_common_bits_mask = (1 << i.bits) - 1;
    let least_common_bits = !most_common_bits & least_common_bits_mask;
    PowerConsumption(most_common_bits * least_common_bits)
}

fn life_support_rating(i: &Input) -> LifeSupportRating {
    let oxygen_generator_rating = oxygen_generator_rating(i);
    let co2_scrubber_rating = co2_scrubber_rating(i);
    LifeSupportRating(oxygen_generator_rating.0 * co2_scrubber_rating.0)
}

fn oxygen_generator_rating(i: &Input) -> OxygenGeneratorRating {
    find_and_build_diags(
        i,
        |(zero_count, one_count)| one_count >= zero_count,
        OxygenGeneratorRating,
    )
}

fn co2_scrubber_rating(i: &Input) -> CO2ScrubberRating {
    find_and_build_diags(
        i,
        |(zero_count, one_count)| zero_count > one_count,
        CO2ScrubberRating,
    )
}

fn find_and_build_diags<F, G, R>(i: &Input, decider: F, build_diag: G) -> R
where
    F: Fn((i32, i32)) -> bool,
    G: Fn(u64) -> R,
{
    let filtered = (0..i.bits)
        .into_iter()
        .fold_while(i.vecs.clone(), |acc, idx| {
            let zero_count_one_count = acc.iter().fold((0, 0), |(zero_count, one_count), vec| {
                // direct vec idx is fine here because Input is parsed and all rows have the same length
                if vec[idx] {
                    (zero_count, one_count + 1)
                } else {
                    (zero_count + 1, one_count)
                }
            });
            let val_to_keep = decider(zero_count_one_count);
            let next: Vec<Vec<bool>> = acc
                .into_iter()
                .filter(|vec| vec[idx] == val_to_keep)
                .collect();
            if next.len() <= 1 {
                FoldWhile::Done(next)
            } else {
                FoldWhile::Continue(next)
            }
        })
        .into_inner();
    let maybe_num = if let Some(v) = filtered.get(0) {
        let num = v.iter().enumerate().fold(0u64, |acc, (idx, next)| {
            if *next {
                let shift = (i.bits - 1 - idx) as u64;
                let mask = 1 << shift;
                acc | mask
            } else {
                acc
            }
        });
        Some(num)
    } else {
        None
    };
    build_diag(maybe_num.unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &str = "00100
11110
10110
10111
10101
01111
00111
11100
10000
11001
00010
01010";

    #[test]
    fn input_parse_test() {
        let r = Input::parse(TEST_INPUT).unwrap();
        let expected = Input {
            vecs: vec![
                vec![false, false, true, false, false],
                vec![true, true, true, true, false],
                vec![true, false, true, true, false],
                vec![true, false, true, true, true],
                vec![true, false, true, false, true],
                vec![false, true, true, true, true],
                vec![false, false, true, true, true],
                vec![true, true, true, false, false],
                vec![true, false, false, false, false],
                vec![true, true, false, false, true],
                vec![false, false, false, true, false],
                vec![false, true, false, true, false],
            ],
            bits: 5,
        };
        assert_eq!(expected, r)
    }

    #[test]
    fn sol_1_test() {
        let input = Input::parse(TEST_INPUT).unwrap();
        let r = power_consumption(&input);
        assert_eq!(198, r.0)
    }

    #[test]
    fn oxygen_generator_rating_test() {
        let input = Input::parse(TEST_INPUT).unwrap();
        let r = oxygen_generator_rating(&input);
        assert_eq!(23, r.0)
    }

    #[test]
    fn co2_scrubber_rating_rating_test() {
        let input = Input::parse(TEST_INPUT).unwrap();
        let r = co2_scrubber_rating(&input);
        assert_eq!(10, r.0)
    }
}
