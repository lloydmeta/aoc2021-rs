use std::result::Result as StdResult;

use anyhow::{Context, Result};
use combine::parser::char::*;
use combine::*;

use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::iter::FromIterator;
use Segment::*;

const INPUT: &str = include_str!("../data/day_08_input");

pub fn run() -> Result<()> {
    println!("*** Day 8: Seven Segment Search ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let sol_1 = unique_number_of_segments_in_output_count(&input);
    println!("Solution 1: {:?}", sol_1);
    let sol_2 = input.solve()?;
    println!("Solution 2: {:?}", sol_2);
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Segments(Vec<Segment>);

#[derive(Debug, Eq, PartialEq)]
struct Line {
    patterns: Vec<Segments>,
    outputs: Vec<Segments>,
}

#[derive(Debug, Eq, PartialEq)]
struct Input(Vec<Line>);

impl Input {
    fn solve(&self) -> Result<usize> {
        let resolved_line_nums = self.0.iter().map(|line| line.resolved_digits_number());
        // manually loop so we capture any errors
        let mut r = 0;
        for resolved_line_num in resolved_line_nums {
            let line_num = resolved_line_num?;
            r += line_num;
        }
        Ok(r)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd)]
enum Segment {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

// Maps the "proper" segments to the current Line's wires
#[derive(Debug, Eq, PartialEq)]
struct OngoingMapping(HashMap<Segment, HashSet<Segment>>);

// Maps the current line's wires segments to the "proper" segments
#[derive(Debug, Eq, PartialEq)]
struct FinishedMapping(HashMap<Segment, Segment>);

impl FinishedMapping {
    fn unmapped_segments_to_number(&self, Segments(unmapped_segments): &Segments) -> Result<usize> {
        let mapped_segments: HashSet<_> = unmapped_segments
            .iter()
            .filter_map(|unmapped_seg| self.0.get(unmapped_seg).copied())
            .collect();
        if mapped_segments.len() != unmapped_segments.len() {
            bail!(
                "Could not map all segments with mappings [{:?}]. Unmapped [{:?}], mapped [{:?}].",
                self.0,
                unmapped_segments,
                mapped_segments
            );
        }
        // stash these somewhere global at one point or a another
        let zero_segments = HashSet::from_iter([A, B, C, E, F, G]);
        let one_segments = HashSet::from_iter([C, F]);
        let two_segments = HashSet::from_iter([A, C, D, E, G]);
        let three_segments = HashSet::from_iter([A, C, D, F, G]);
        let four_segments = HashSet::from_iter([B, C, D, F]);
        let five_segments = HashSet::from_iter([A, B, D, F, G]);
        let six_segments = HashSet::from_iter([A, B, D, E, F, G]);
        let seven_segments = HashSet::from_iter([A, C, F]);
        let eight_segments = HashSet::from_iter([A, B, C, D, E, F, G]);
        let nine_segments = HashSet::from_iter([A, B, C, D, F, G]);

        let number = if mapped_segments == zero_segments {
            0
        } else if mapped_segments == one_segments {
            1
        } else if mapped_segments == two_segments {
            2
        } else if mapped_segments == three_segments {
            3
        } else if mapped_segments == four_segments {
            4
        } else if mapped_segments == five_segments {
            5
        } else if mapped_segments == six_segments {
            6
        } else if mapped_segments == seven_segments {
            7
        } else if mapped_segments == eight_segments {
            8
        } else if mapped_segments == nine_segments {
            9
        } else {
            bail!(
                "Could not map segments [{:?}] to anything using mappings [{:?}]",
                unmapped_segments,
                self.0
            )
        };
        Ok(number)
    }
}

impl OngoingMapping {
    fn unsolved() -> OngoingMapping {
        let mut all_possible_segments = HashSet::with_capacity(7);
        all_possible_segments.insert(A);
        all_possible_segments.insert(B);
        all_possible_segments.insert(C);
        all_possible_segments.insert(D);
        all_possible_segments.insert(E);
        all_possible_segments.insert(F);
        all_possible_segments.insert(G);
        let mut unsolved_mapping = HashMap::with_capacity(7);
        unsolved_mapping.insert(A, all_possible_segments.clone());
        unsolved_mapping.insert(B, all_possible_segments.clone());
        unsolved_mapping.insert(C, all_possible_segments.clone());
        unsolved_mapping.insert(D, all_possible_segments.clone());
        unsolved_mapping.insert(E, all_possible_segments.clone());
        unsolved_mapping.insert(F, all_possible_segments.clone());
        unsolved_mapping.insert(G, all_possible_segments.clone());

        OngoingMapping(unsolved_mapping)
    }

    fn into_finished(self) -> Result<FinishedMapping> {
        let mut m = HashMap::with_capacity(7);
        for (proper_segment, segments_mapped_to) in self.0 {
            if !segments_mapped_to.len() == 1 {
                bail!(
                    "Found non-singular mapping for [{:?}]: [{:?}]",
                    proper_segment,
                    segments_mapped_to
                );
            }
            if let Some(segment_mapped_to) = segments_mapped_to.into_iter().next() {
                m.insert(segment_mapped_to, proper_segment);
            }
        }
        Ok(FinishedMapping(m))
    }
}

impl Display for OngoingMapping {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for (mapping, segments) in &self.0 {
            writeln!(f, "{:?}: {:?}", mapping, segments)?;
        }
        write!(f, "}}")
    }
}

fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
    let line_parser = sep_by1(segments_parser(), char(' '))
        .skip(string("| "))
        .and(sep_by1(segments_parser(), char(' ')))
        .map(|(patterns, outputs): (Vec<Segments>, Vec<Segments>)| {
            /* ugly but at least I'm still using parser combinators ... */
            let non_empty_patterns = patterns
                .into_iter()
                .filter(|segs| !segs.0.is_empty())
                .map(|mut segs| {
                    segs.0.sort_by_key(|s| *s);
                    segs
                })
                .sorted_by_key(|segs| segs.0.len())
                .collect();
            let non_empty_outputs = outputs
                .into_iter()
                .filter(|segs| !segs.0.is_empty())
                .map(|mut segs| {
                    segs.0.sort_by_key(|s| *s);
                    segs
                })
                .collect();
            Line {
                patterns: non_empty_patterns,
                outputs: non_empty_outputs,
            }
        });
    let mut parser = many1(line_parser.skip(spaces())).map(Input);
    let (r, _) = parser.easy_parse(s)?;
    Ok(r)
}

// Count numbers that can be 1, 4, 7, or 8 since they have unique segment counts
fn unique_number_of_segments_in_output_count(i: &Input) -> usize {
    i.0.iter().fold(0, |acc, Line { outputs, .. }| {
        outputs
            .iter()
            .filter(|seg| {
                let segment_count = seg.0.len();
                matches!(segment_count, 2 | 3 | 4 | 7)
            })
            .count()
            + acc
    })
}

impl Line {
    fn resolved_digits_number(&self) -> Result<usize> {
        let output_digits_count = self.outputs.len();
        let final_mappings = self.create_mappings()?.into_finished()?;
        let mut digits = Vec::with_capacity(output_digits_count);
        for unresolved_output_segment in &self.outputs {
            digits.push(final_mappings.unmapped_segments_to_number(unresolved_output_segment)?)
        }
        let final_number = digits
            .iter()
            .enumerate()
            .fold(0usize, |acc, (digit_idx, digit)| {
                let multiplier = 10_usize.pow((output_digits_count - 1 - digit_idx) as u32);
                acc + multiplier * digit
            });
        Ok(final_number)
    }

    fn create_mappings(&self) -> Result<OngoingMapping> {
        let mut ongoing_mapping = OngoingMapping::unsolved();
        // Part 1:
        for Segments(segments_in_pattern) in &self.patterns {
            if segments_in_pattern.len() == 2 {
                // this is a 1, so c and f must map to it somehow
                // remove from all other mappings
                for (mapping_segment, possible_segments_to_map_to) in ongoing_mapping.0.iter_mut() {
                    if *mapping_segment == C || *mapping_segment == F {
                        possible_segments_to_map_to.retain(|s| segments_in_pattern.contains(s));
                    } else {
                        possible_segments_to_map_to.retain(|s| !segments_in_pattern.contains(s));
                    }
                }
            }
            if segments_in_pattern.len() == 3 {
                // this is a 7, so c and f must map to it somehow, and the _other_ one is a (top)
                // remove from all other mappings
                // since we're processing in order, we know c_or_f is going to be defined as just the two
                let maybe_c_or_f_segments = ongoing_mapping
                    .0
                    .get(&C)
                    .or_else(|| ongoing_mapping.0.get(&F))
                    .cloned(); // not proud of this clone...
                for (mapping_segment, possible_segments_to_map_to) in ongoing_mapping.0.iter_mut() {
                    if *mapping_segment == C || *mapping_segment == F {
                        possible_segments_to_map_to.retain(|s| segments_in_pattern.contains(s));
                    } else if *mapping_segment == A {
                        possible_segments_to_map_to.retain(|s| {
                            segments_in_pattern.contains(s)
                                && !maybe_c_or_f_segments
                                    .as_ref()
                                    .map(|c_or_f| c_or_f.contains(s))
                                    .unwrap_or(false)
                        });

                        // At this point, 'A' mapping should be unambiguous (single possible mapping)
                    } else {
                        possible_segments_to_map_to.retain(|s| !segments_in_pattern.contains(s));
                    }
                }
            }
            if segments_in_pattern.len() == 4 {
                // this is a 4, so b, d, c and f must map to it somehow, but we already know what
                // c or f must be
                let c_or_f_segments = ongoing_mapping
                    .0
                    .get(&C)
                    .or_else(|| ongoing_mapping.0.get(&F))
                    .cloned() // not proud of this clone...
                    .with_context(|| "C or F mappings should be defined ")?;
                for (mapping_segment, possible_segments_to_map_to) in ongoing_mapping.0.iter_mut() {
                    if *mapping_segment == C || *mapping_segment == F {
                        possible_segments_to_map_to.retain(|s| segments_in_pattern.contains(s));
                    } else if *mapping_segment == B || *mapping_segment == D {
                        possible_segments_to_map_to.retain(|s| {
                            segments_in_pattern.contains(s) && !c_or_f_segments.contains(s)
                        });
                        // At this point, 'B' and `D` mapping should be 2 possible values
                    } else {
                        possible_segments_to_map_to.retain(|s| !segments_in_pattern.contains(s));
                    }
                }
            }
        }
        // now, we can solve for 3: it has:
        // - 5 segments
        // - c_and_f
        // - a segment (they all do though)
        // - g segment (they all do though)
        // - allows us to know definitively what *d* mapping is is between b_and_d, which means we know what *b* is,
        //    since it has D but not B
        // this is a 4, so b, d, c and f must map to it somehow, but we already know what
        // c or f must be
        let c_or_f_segments = ongoing_mapping
            .0
            .get(&C)
            .or_else(|| ongoing_mapping.0.get(&F))
            .cloned() // not proud of this clone...
            .with_context(|| "C or F mappings should be defined ")?;

        for Segments(segments_in_pattern) in &self.patterns {
            if segments_in_pattern.len() == 5
                && !c_or_f_segments.is_empty()
                && c_or_f_segments
                    .iter()
                    .all(|c_or_f_segment| segments_in_pattern.contains(c_or_f_segment))
            {
                // 3 does have *D* segment, but not B
                for (mapping_segment, possible_segments_to_map_to) in ongoing_mapping.0.iter_mut() {
                    if *mapping_segment == B {
                        possible_segments_to_map_to
                            .retain(|b_segment| !segments_in_pattern.contains(b_segment));
                    } else if *mapping_segment == D {
                        possible_segments_to_map_to
                            .retain(|d_segment| segments_in_pattern.contains(d_segment));
                    }
                }
            }
        }
        // By now, A, B and D mappings should be unambiguous while c_and_f still has two possible entries
        // this allows us to use 5:
        //  - 5 segments
        //  - has b segment (unique among 5 segs)
        //  - has f and g segment
        // // to  solve for
        //  -  `f` and thus also `c`
        //  -  `g`: at this point we know what everything else is in this pattern
        let a_segments = ongoing_mapping
            .0
            .get(&A)
            .cloned() // not proud of this clone...
            .with_context(|| "A mappings should be defined")?;
        let b_segments = ongoing_mapping
            .0
            .get(&B)
            .cloned() // not proud of this clone...
            .with_context(|| "B mappings should be defined")?;
        let d_segments = ongoing_mapping
            .0
            .get(&D)
            .cloned() // not proud of this clone...
            .with_context(|| "D mappings should be defined")?;

        for Segments(segments_in_pattern) in &self.patterns {
            if segments_in_pattern.len() == 5
                && !b_segments.is_empty()
                && b_segments
                    .iter()
                    .all(|b_segment| segments_in_pattern.contains(b_segment))
            {
                // 3 does have *D* segment, but not B
                for (mapping_segment, possible_segments_to_map_to) in ongoing_mapping.0.iter_mut() {
                    if *mapping_segment == F {
                        possible_segments_to_map_to
                            .retain(|f_segment| segments_in_pattern.contains(f_segment));
                    } else if *mapping_segment == C {
                        possible_segments_to_map_to
                            .retain(|c_segment| !segments_in_pattern.contains(c_segment));
                    } else if *mapping_segment == G {
                        possible_segments_to_map_to.retain(|g_segment| {
                            !(a_segments.contains(g_segment)
                                || b_segments.contains(g_segment)
                                || d_segments.contains(g_segment)
                                || c_or_f_segments.contains(g_segment))
                                && segments_in_pattern.contains(g_segment)
                        });
                    }
                }
            }
        }
        // At this point, we should have mapped out a, b, c, d, f, and g, so e is just the other one.
        let c_segments = ongoing_mapping
            .0
            .get(&C)
            .cloned() // not proud of this clone...
            .with_context(|| "C mappings should be defined ")?;
        let f_segments = ongoing_mapping
            .0
            .get(&F)
            .cloned() // not proud of this clone...
            .with_context(|| "F mappings should be defined ")?;
        let g_segments = ongoing_mapping
            .0
            .get(&G)
            .cloned() // not proud of this clone...
            .with_context(|| "G mappings should be defined ")?;
        if let Some(e_mappings) = ongoing_mapping.0.get_mut(&E) {
            e_mappings.retain(|e_segment| {
                !(a_segments.contains(e_segment)
                    || b_segments.contains(e_segment)
                    || c_segments.contains(e_segment)
                    || d_segments.contains(e_segment)
                    || f_segments.contains(e_segment)
                    || g_segments.contains(e_segment))
            })
        }

        Ok(ongoing_mapping)
    }
}

fn segments_parser<Input>() -> impl Parser<Input, Output = Segments>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    <Input as combine::StreamOnce>::Error: combine::ParseError<
        char,
        <Input as combine::StreamOnce>::Range,
        <Input as combine::StreamOnce>::Position,
    >,
{
    many(choice![
        attempt(char('a').map(|_| A)),
        attempt(char('b').map(|_| B)),
        attempt(char('c').map(|_| C)),
        attempt(char('d').map(|_| D)),
        attempt(char('e').map(|_| E)),
        attempt(char('f').map(|_| F)),
        attempt(char('g').map(|_| G))
    ])
    .map(Segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &str =
        "be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe cefdb cefbgd gcbe
edbfga begcd cbg gc gcadebf fbgde acbgfd abcde gfcbed gfec | fcgedb cgb dgebacf gc
fgaebd cg bdaec gdafb agbcfd gdcbef bgcad gfac gcb cdgabef | cg cg fdcagb cbg
fbegcd cbd adcefb dageb afcb bc aefdc ecdab fgdeca fcdbega | efabcd cedba gadfec cb
aecbfdg fbg gf bafeg dbefa fcge gcbea fcaegb dgceab fcbdga | gecf egdcabf bgf bfgea
fgeab ca afcebg bdacfeg cfaedg gcfdb baec bfadeg bafgc acf | gebdcfa ecba ca fadegcb
dbcfg fgd bdegcaf fgec aegbdf ecdfab fbedc dacgb gdcebf gf | cefg dcbef fcge gbcadfe
bdfegc cbegaf gecbf dfcage bdacg ed bedf ced adcbefg gebcd | ed bcgafe cdgba cbgef
egadfb cdbfeg cegd fecab cgb gbdefca cg fgcdab egfdb bfceg | gbdfcae bgc cg cgb
gcafb gcf dcaebfg ecagb gf abcdeg gaef cafbge fdbac fegbdc | fgae cfgab fg bagce";

    static TEST_INPUT_SINGLE: &str =
        "acedgfb cdfbe gcdfa fbcad dab cefabd cdfgeb eafb cagedb ab | cdfeb fcadb cdfeb cdbaf";

    #[test]
    fn unique_number_of_segments_in_output_count_test() {
        let r = parse(TEST_INPUT).unwrap();
        let s = unique_number_of_segments_in_output_count(&r);
        assert_eq!(26, s);
    }

    #[test]
    fn line_resolve_mappings_test() {
        let r = parse(TEST_INPUT_SINGLE).unwrap();
        let s: Vec<_> =
            r.0.iter()
                .filter_map(|line| line.create_mappings().ok())
                .collect();
        println!("s [{:?}]", s);
    }

    #[test]
    fn input_solve_test() {
        let i = parse(TEST_INPUT).unwrap();
        let r = i.solve().unwrap();
        assert_eq!(61229, r);
    }

    #[test]
    fn resolved_digits_number_single_test() {
        let r = parse(TEST_INPUT_SINGLE).unwrap();
        let s: Vec<_> =
            r.0.iter()
                .filter_map(|line| line.resolved_digits_number().ok())
                .collect();
        assert_eq!(Some(5353usize), s.first().map(ToOwned::to_owned))
    }

    #[test]
    fn resolved_digits_number_single_multiple() {
        let r = parse(TEST_INPUT).unwrap();
        let s: Vec<_> =
            r.0.iter()
                .filter_map(|line| line.resolved_digits_number().ok())
                .collect();
        println!("Resolved [{:?}]", s);
    }

    #[test]
    fn input_parse_test() {
        let r = parse(TEST_INPUT).unwrap();
        let expected = Input(vec![
            Line {
                patterns: vec![
                    Segments(vec![B, E]),
                    Segments(vec![B, D, E]),
                    Segments(vec![B, C, E, G]),
                    Segments(vec![C, D, E, F, G]),
                    Segments(vec![B, C, D, E, F]),
                    Segments(vec![A, B, C, D, F]),
                    Segments(vec![B, C, D, E, F, G]),
                    Segments(vec![A, C, D, E, F, G]),
                    Segments(vec![A, B, D, E, F, G]),
                    Segments(vec![A, B, C, D, E, F, G]),
                ],
                outputs: vec![
                    Segments(vec![A, B, C, D, E, F, G]),
                    Segments(vec![B, C, D, E, F]),
                    Segments(vec![B, C, D, E, F, G]),
                    Segments(vec![B, C, E, G]),
                ],
            },
            Line {
                patterns: vec![
                    Segments(vec![C, G]),
                    Segments(vec![B, C, G]),
                    Segments(vec![C, E, F, G]),
                    Segments(vec![B, C, D, E, G]),
                    Segments(vec![B, D, E, F, G]),
                    Segments(vec![A, B, C, D, E]),
                    Segments(vec![A, B, D, E, F, G]),
                    Segments(vec![A, B, C, D, F, G]),
                    Segments(vec![B, C, D, E, F, G]),
                    Segments(vec![A, B, C, D, E, F, G]),
                ],
                outputs: vec![
                    Segments(vec![B, C, D, E, F, G]),
                    Segments(vec![B, C, G]),
                    Segments(vec![A, B, C, D, E, F, G]),
                    Segments(vec![C, G]),
                ],
            },
            Line {
                patterns: vec![
                    Segments(vec![C, G]),
                    Segments(vec![B, C, G]),
                    Segments(vec![A, C, F, G]),
                    Segments(vec![A, B, C, D, E]),
                    Segments(vec![A, B, D, F, G]),
                    Segments(vec![A, B, C, D, G]),
                    Segments(vec![A, B, D, E, F, G]),
                    Segments(vec![A, B, C, D, F, G]),
                    Segments(vec![B, C, D, E, F, G]),
                    Segments(vec![A, B, C, D, E, F, G]),
                ],
                outputs: vec![
                    Segments(vec![C, G]),
                    Segments(vec![C, G]),
                    Segments(vec![A, B, C, D, F, G]),
                    Segments(vec![B, C, G]),
                ],
            },
            Line {
                patterns: vec![
                    Segments(vec![B, C]),
                    Segments(vec![B, C, D]),
                    Segments(vec![A, B, C, F]),
                    Segments(vec![A, B, D, E, G]),
                    Segments(vec![A, C, D, E, F]),
                    Segments(vec![A, B, C, D, E]),
                    Segments(vec![B, C, D, E, F, G]),
                    Segments(vec![A, B, C, D, E, F]),
                    Segments(vec![A, C, D, E, F, G]),
                    Segments(vec![A, B, C, D, E, F, G]),
                ],
                outputs: vec![
                    Segments(vec![A, B, C, D, E, F]),
                    Segments(vec![A, B, C, D, E]),
                    Segments(vec![A, C, D, E, F, G]),
                    Segments(vec![B, C]),
                ],
            },
            Line {
                patterns: vec![
                    Segments(vec![F, G]),
                    Segments(vec![B, F, G]),
                    Segments(vec![C, E, F, G]),
                    Segments(vec![A, B, E, F, G]),
                    Segments(vec![A, B, D, E, F]),
                    Segments(vec![A, B, C, E, G]),
                    Segments(vec![A, B, C, E, F, G]),
                    Segments(vec![A, B, C, D, E, G]),
                    Segments(vec![A, B, C, D, F, G]),
                    Segments(vec![A, B, C, D, E, F, G]),
                ],
                outputs: vec![
                    Segments(vec![C, E, F, G]),
                    Segments(vec![A, B, C, D, E, F, G]),
                    Segments(vec![B, F, G]),
                    Segments(vec![A, B, E, F, G]),
                ],
            },
            Line {
                patterns: vec![
                    Segments(vec![A, C]),
                    Segments(vec![A, C, F]),
                    Segments(vec![A, B, C, E]),
                    Segments(vec![A, B, E, F, G]),
                    Segments(vec![B, C, D, F, G]),
                    Segments(vec![A, B, C, F, G]),
                    Segments(vec![A, B, C, E, F, G]),
                    Segments(vec![A, C, D, E, F, G]),
                    Segments(vec![A, B, D, E, F, G]),
                    Segments(vec![A, B, C, D, E, F, G]),
                ],
                outputs: vec![
                    Segments(vec![A, B, C, D, E, F, G]),
                    Segments(vec![A, B, C, E]),
                    Segments(vec![A, C]),
                    Segments(vec![A, B, C, D, E, F, G]),
                ],
            },
            Line {
                patterns: vec![
                    Segments(vec![F, G]),
                    Segments(vec![D, F, G]),
                    Segments(vec![C, E, F, G]),
                    Segments(vec![B, C, D, F, G]),
                    Segments(vec![B, C, D, E, F]),
                    Segments(vec![A, B, C, D, G]),
                    Segments(vec![A, B, D, E, F, G]),
                    Segments(vec![A, B, C, D, E, F]),
                    Segments(vec![B, C, D, E, F, G]),
                    Segments(vec![A, B, C, D, E, F, G]),
                ],
                outputs: vec![
                    Segments(vec![C, E, F, G]),
                    Segments(vec![B, C, D, E, F]),
                    Segments(vec![C, E, F, G]),
                    Segments(vec![A, B, C, D, E, F, G]),
                ],
            },
            Line {
                patterns: vec![
                    Segments(vec![D, E]),
                    Segments(vec![C, D, E]),
                    Segments(vec![B, D, E, F]),
                    Segments(vec![B, C, E, F, G]),
                    Segments(vec![A, B, C, D, G]),
                    Segments(vec![B, C, D, E, G]),
                    Segments(vec![B, C, D, E, F, G]),
                    Segments(vec![A, B, C, E, F, G]),
                    Segments(vec![A, C, D, E, F, G]),
                    Segments(vec![A, B, C, D, E, F, G]),
                ],
                outputs: vec![
                    Segments(vec![D, E]),
                    Segments(vec![A, B, C, E, F, G]),
                    Segments(vec![A, B, C, D, G]),
                    Segments(vec![B, C, E, F, G]),
                ],
            },
            Line {
                patterns: vec![
                    Segments(vec![C, G]),
                    Segments(vec![B, C, G]),
                    Segments(vec![C, D, E, G]),
                    Segments(vec![A, B, C, E, F]),
                    Segments(vec![B, D, E, F, G]),
                    Segments(vec![B, C, E, F, G]),
                    Segments(vec![A, B, D, E, F, G]),
                    Segments(vec![B, C, D, E, F, G]),
                    Segments(vec![A, B, C, D, F, G]),
                    Segments(vec![A, B, C, D, E, F, G]),
                ],
                outputs: vec![
                    Segments(vec![A, B, C, D, E, F, G]),
                    Segments(vec![B, C, G]),
                    Segments(vec![C, G]),
                    Segments(vec![B, C, G]),
                ],
            },
            Line {
                patterns: vec![
                    Segments(vec![F, G]),
                    Segments(vec![C, F, G]),
                    Segments(vec![A, E, F, G]),
                    Segments(vec![A, B, C, F, G]),
                    Segments(vec![A, B, C, E, G]),
                    Segments(vec![A, B, C, D, F]),
                    Segments(vec![A, B, C, D, E, G]),
                    Segments(vec![A, B, C, E, F, G]),
                    Segments(vec![B, C, D, E, F, G]),
                    Segments(vec![A, B, C, D, E, F, G]),
                ],
                outputs: vec![
                    Segments(vec![A, E, F, G]),
                    Segments(vec![A, B, C, F, G]),
                    Segments(vec![F, G]),
                    Segments(vec![A, B, C, E, G]),
                ],
            },
        ]);
        assert_eq!(expected, r);
    }
}
