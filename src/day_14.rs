use std::collections::HashMap;
use std::result::Result as StdResult;

use anyhow::Result;
use combine::parser::char::*;
use combine::*;

pub const INPUT: &str = include_str!("../data/day_14_input");

const DUMMY_CHAR: char = '!';

pub fn run() -> Result<()> {
    println!("*** Day 14: Extended Polymerization ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;

    let sol_1 = input.max_minus_least_after_steps(10);
    println!("Solution 1: {:?}", sol_1);

    let sol_2 = input.max_minus_least_after_steps(40);
    println!("Solution 2: {:?}", sol_2);

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub struct Input {
    template: Template,
    mappings: HashMap<Pair, char>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Template(Vec<char>);

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct Pair {
    first: char,
    second: char,
}

impl Input {
    pub fn max_minus_least_after_steps(&self, steps: usize) -> Option<usize> {
        let pairs_to_counts = self.pair_wise_counts(steps);
        let chars_to_double_counts = pairs_to_counts.iter().fold(
            HashMap::new(),
            |mut acc, (Pair { first, second }, count)| {
                *acc.entry(first).or_insert(0) += count;
                *acc.entry(second).or_insert(0) += count;
                acc
            },
        );
        let maybe_least_most = chars_to_double_counts
            .iter()
            .filter_map(|(c, double_count)| {
                if **c != DUMMY_CHAR {
                    Some((c, double_count / 2))
                } else {
                    None
                }
            })
            .fold(None, |acc, (c, count)| {
                acc.map(|((least_char, least_count), (most_char, most_count))| {
                    if count < least_count {
                        ((c, count), (most_char, most_count))
                    } else if count > most_count {
                        ((least_char, least_count), (c, count))
                    } else {
                        ((least_char, least_count), (most_char, most_count))
                    }
                })
                .or(Some(((c, count), (c, count))))
            });
        // ABA -> _A, AB, BA, A_ -> A:4, B: 2 -> a:2 , b:1
        maybe_least_most.map(|((_least_char, least_count), (_most_char, most_count))| {
            most_count - least_count + 1
        })
    }

    fn pair_wise_counts(&self, times: usize) -> HashMap<Pair, usize> {
        let mut init_pair_counts =
            self.template
                .0
                .as_slice()
                .windows(2)
                .fold(HashMap::new(), |mut acc, window_pair| {
                    let first = window_pair[0];
                    let second = window_pair[1];
                    let pair = Pair { first, second };
                    *acc.entry(pair).or_insert(0) += 1;
                    acc
                });
        // insert first *and* last dummy pair while stepping so we can divide by two evenly
        if let Some(first_char) = self.template.0.first() {
            let pair = Pair {
                first: DUMMY_CHAR,
                second: *first_char,
            };
            *init_pair_counts.entry(pair).or_insert(0) += 1;
        }
        if let Some(last_char) = self.template.0.last() {
            let pair = Pair {
                first: *last_char,
                second: DUMMY_CHAR,
            };
            *init_pair_counts.entry(pair).or_insert(0) += 1;
        }

        (0..times).fold(init_pair_counts, |previous_acc, _time| {
            previous_acc.iter().fold(
                HashMap::with_capacity(previous_acc.len()),
                |mut new_acc, (pair, count)| {
                    let mapped = self.mappings.get(pair);
                    if let Some(mapped_char) = mapped {
                        let first_new_pair = Pair {
                            first: pair.first,
                            second: *mapped_char,
                        };
                        let second_new_pair = Pair {
                            first: *mapped_char,
                            second: pair.second,
                        };
                        *new_acc.entry(first_new_pair).or_insert(0) += count;
                        *new_acc.entry(second_new_pair).or_insert(0) += count;
                    }
                    new_acc
                },
            )
        })
    }
}

pub fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
    let template_parser = many1(upper()).map(Template);
    let mapping_parser = upper()
        .and(upper())
        .map(|(first, second)| Pair { first, second })
        .skip(string(" -> "))
        .and(upper());
    let mappings_parser = many1(mapping_parser.skip(spaces()))
        .map(|mappings: Vec<(Pair, char)>| mappings.into_iter().collect::<HashMap<Pair, char>>());

    let mut parser = template_parser
        .skip(spaces())
        .and(mappings_parser)
        .map(|(template, mappings)| Input { template, mappings });
    let (r, _) = parser.easy_parse(s)?;
    Ok(r)
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use super::*;

    static TEST_INPUT: &str = "NNCB

CH -> B
HH -> N
CB -> H
NH -> C
HB -> C
HC -> B
HN -> C
NN -> C
BH -> H
NC -> B
NB -> B
BN -> B
BB -> N
BC -> B
CC -> N
CN -> C";

    #[test]
    fn sol_1_test() {
        let i = parse(TEST_INPUT).unwrap();
        let r = i.max_minus_least_after_steps(10).unwrap();
        assert_eq!(1588, r)
    }

    #[test]
    fn part_1_test() {
        let i = parse(INPUT).unwrap();
        let r = i.max_minus_least_after_steps(10).unwrap();
        assert_eq!(2587, r)
    }

    #[test]
    fn part_2_test() {
        let i = parse(INPUT).unwrap();
        let r = i.max_minus_least_after_steps(40).unwrap();
        assert_eq!(3318837563123, r)
    }

    #[test]
    fn parse_test() {
        let i = parse(TEST_INPUT).unwrap();
        let expected = Input {
            template: Template(vec!['N', 'N', 'C', 'B']),
            mappings: HashMap::from_iter([
                (
                    Pair {
                        first: 'C',
                        second: 'N',
                    },
                    'C',
                ),
                (
                    Pair {
                        first: 'N',
                        second: 'H',
                    },
                    'C',
                ),
                (
                    Pair {
                        first: 'C',
                        second: 'H',
                    },
                    'B',
                ),
                (
                    Pair {
                        first: 'B',
                        second: 'H',
                    },
                    'H',
                ),
                (
                    Pair {
                        first: 'H',
                        second: 'N',
                    },
                    'C',
                ),
                (
                    Pair {
                        first: 'N',
                        second: 'N',
                    },
                    'C',
                ),
                (
                    Pair {
                        first: 'N',
                        second: 'B',
                    },
                    'B',
                ),
                (
                    Pair {
                        first: 'N',
                        second: 'C',
                    },
                    'B',
                ),
                (
                    Pair {
                        first: 'H',
                        second: 'B',
                    },
                    'C',
                ),
                (
                    Pair {
                        first: 'B',
                        second: 'N',
                    },
                    'B',
                ),
                (
                    Pair {
                        first: 'C',
                        second: 'C',
                    },
                    'N',
                ),
                (
                    Pair {
                        first: 'H',
                        second: 'H',
                    },
                    'N',
                ),
                (
                    Pair {
                        first: 'C',
                        second: 'B',
                    },
                    'H',
                ),
                (
                    Pair {
                        first: 'B',
                        second: 'C',
                    },
                    'B',
                ),
                (
                    Pair {
                        first: 'B',
                        second: 'B',
                    },
                    'N',
                ),
                (
                    Pair {
                        first: 'H',
                        second: 'C',
                    },
                    'B',
                ),
            ]),
        };
        assert_eq!(expected, i)
    }
}
