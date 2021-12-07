use std::collections::HashMap;
use std::convert::identity;
use std::result::Result as StdResult;

use anyhow::Result;
use combine::parser::char::*;
use combine::*;
use itertools::Itertools;

use crate::common::usize_parser;

const INPUT: &str = include_str!("../data/day_07_input");

pub fn run() -> Result<()> {
    println!("*** Day 7: The Treachery of Whales ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let sol_1 = solve_part_1(&input);
    println!("Solution 1: {:?}", sol_1);
    let sol_2 = solve_part_2(&input);
    println!("Solution 2: {:?}", sol_2);
    Ok(())
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct Pos(usize);

#[derive(Debug, Eq, PartialEq)]
struct TotalFuelCost(usize);

#[derive(Debug, Eq, PartialEq)]
struct Input(Vec<Pos>);

fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
    let mut parser = sep_by1(usize_parser().map(Pos), char(',')).map(Input);
    let (r, _) = parser.easy_parse(s)?;
    Ok(r)
}

fn solve_part_1(i: &Input) -> Option<(Pos, TotalFuelCost)> {
    cheapest_fuel_cost(i, identity)
}

fn solve_part_2(i: &Input) -> Option<(Pos, TotalFuelCost)> {
    cheapest_fuel_cost(i, |dist| {
        // Gauss
        ((dist as f32 / 2f32) * (dist as f32 + 1f32)) as usize
    })
}

fn cheapest_fuel_cost<F>(i: &Input, distance_to_fuel_cost: F) -> Option<(Pos, TotalFuelCost)>
where
    F: Fn(usize) -> usize,
{
    let ((min_pos, max_pos), pos_to_counts) = i.0.iter().fold(
        ((Pos(0), Pos(0)), HashMap::with_capacity(i.0.len())),
        |((min_pos_acc, max_pos_acc), mut pos_to_count_acc), next| {
            let next_min = min_pos_acc.0.min(next.0);
            let next_max = max_pos_acc.0.max(next.0);
            *pos_to_count_acc.entry(next).or_insert(0) += 1;
            ((Pos(next_min), Pos(next_max)), pos_to_count_acc)
        },
    );

    let mut positions_to_shift_to_to_fuel_costs = (min_pos.0..=max_pos.0)
        .map(|pos_candidate| {
            let total_fuel_cost_for_pos_candidate =
                pos_to_counts.iter().fold(0, |acc, (pos, crab_counts)| {
                    let distance_from_candidate =
                        (pos.0 as isize - pos_candidate as isize).abs() as usize;
                    let total_fuel_cost =
                        distance_to_fuel_cost(distance_from_candidate) * crab_counts;
                    acc + total_fuel_cost
                });
            (
                Pos(pos_candidate),
                TotalFuelCost(total_fuel_cost_for_pos_candidate),
            )
        })
        .sorted_by_key(|(_, fuel_costs)| fuel_costs.0);
    positions_to_shift_to_to_fuel_costs.next()
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &str = "16,1,2,0,4,2,7,1,2,14";

    #[test]
    fn input_parse_test() {
        let r = parse(TEST_INPUT).unwrap();
        let expected = Input(vec![
            Pos(16),
            Pos(1),
            Pos(2),
            Pos(0),
            Pos(4),
            Pos(2),
            Pos(7),
            Pos(1),
            Pos(2),
            Pos(14),
        ]);
        assert_eq!(expected, r);
    }

    #[test]
    fn solve_part_1_test() {
        let r = parse(TEST_INPUT).unwrap();
        let s = solve_part_1(&r);
        assert_eq!(Some((Pos(2), TotalFuelCost(37))), s);
    }

    #[test]
    fn solve_part_2_test() {
        let r = parse(TEST_INPUT).unwrap();
        let s = solve_part_2(&r);
        assert_eq!(Some((Pos(5), TotalFuelCost(168))), s);
    }
}
