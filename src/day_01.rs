use anyhow::Result;
use itertools::*;
use std::cmp::Ordering;
use std::cmp::Ordering::*;

const INPUT: &str = include_str!("../data/day_01_input");

pub fn run() -> Result<()> {
    println!("*** Day 1: Sonar Sweep ***");
    println!("Input: {}", INPUT);
    let nums = string_to_digits(INPUT);
    let changes = to_changes(&nums);
    let increases = count_increases(changes);
    println!("Solution 1: {:?}\n", increases);
    let three_measurement_window_sums = nums
        .windows(3)
        .map(|window| window.iter().sum())
        .collect::<Vec<isize>>();
    let changes = to_changes(&three_measurement_window_sums);
    let increases = count_increases(changes);
    println!("Solution 2: {:?}\n", increases);
    Ok(())
}

fn string_to_digits(s: &str) -> Vec<isize> {
    s.split('\n').filter_map(|v| v.parse().ok()).collect()
}

fn to_changes(v: &[isize]) -> impl Iterator<Item = Ordering> + '_ {
    v.iter().zip(v.iter().dropping(1)).map(|(x, y)| y.cmp(x))
}

fn count_increases(v: impl Iterator<Item = Ordering>) -> usize {
    v.filter(|c| *c == Greater).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_to_digits_test() {
        assert_eq!(string_to_digits("1234\n5432"), vec![1234, 5432]);
    }

    #[test]
    fn count_larger_than_previous_test() {
        let r = to_changes(&[1234, 5432, 1, 1]).collect::<Vec<Ordering>>();
        assert_eq!(vec![Greater, Less, Equal], r);
    }
}
