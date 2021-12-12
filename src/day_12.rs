use std::collections::*;
use std::result::Result as StdResult;

use anyhow::{Context, Result};
use combine::parser::char::*;
use combine::*;
use itertools::Itertools;

use Point::*;

const INPUT: &str = include_str!("../data/day_12_input");
const THRESHOLD_FOR_REPEATABLE: usize = 2;

pub fn run() -> Result<()> {
    println!("*** Day 12: Passage Pathing ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let sol_1 = generate_paths(&input, false)?;
    println!("Solution 1: {:?}", sol_1.len());
    let sol_2 = generate_paths(&input, true)?;
    println!("Solution 2: {:?}", sol_2.len());
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Input(HashMap<Point, HashSet<Point>>);

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
enum Point {
    Start,
    BigCave(String),
    SmallCave(String),
    End,
}

fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
    macro_rules! point_parser {
        () => {
            many1(letter()).map(|s: String| {
                if s == "start" {
                    Point::Start
                } else if s == "end" {
                    Point::End
                } else {
                    if s.chars().all(|c| c.is_uppercase()) {
                        Point::BigCave(s)
                    } else {
                        Point::SmallCave(s)
                    }
                }
            })
        };
    }
    let line_parser = point_parser!().skip(char('-')).and(point_parser!());
    let mut parser = many1(line_parser.skip(spaces())).map(|lines: Vec<(Point, Point)>| {
        let mut connections = HashMap::with_capacity(lines.len());
        for (from, to) in lines.into_iter() {
            connections
                .entry(from.clone())
                .or_insert_with(HashSet::new)
                .insert(to.clone());
            connections
                .entry(to)
                .or_insert_with(HashSet::new)
                .insert(from);
        }
        Input(connections)
    });

    let (r, _) = parser.easy_parse(s)?;
    Ok(r)
}

fn generate_paths(
    Input(connections): &Input,
    single_small_cave_repeat: bool,
) -> Result<Vec<Vec<&Point>>> {
    let initial_points_from_start = connections.get(&Start).context("No Start point found!")?;
    let build_paths_with_maybe_small_cave_to_repeat =
        |maybe_small_cave_to_repeat: Option<&Point>| {
            initial_points_from_start
                .iter()
                .flat_map(|point| {
                    let mut reverse_paths = generate_reverse_sub_paths(
                        connections,
                        point,
                        maybe_small_cave_to_repeat,
                        HashMap::new(),
                    );
                    reverse_paths.iter_mut().for_each(|path| {
                        path.push(&Start);
                        path.reverse()
                    });
                    reverse_paths // no longer reversed
                })
                .unique()
                .collect()
        };
    let paths = if single_small_cave_repeat {
        let small_caves = connections
            .keys()
            .filter(|p| matches!(p, Point::SmallCave(_)));
        small_caves
            .flat_map(|small_cave_to_repeat| {
                build_paths_with_maybe_small_cave_to_repeat(Some(small_cave_to_repeat))
            })
            .unique()
            .collect()
    } else {
        build_paths_with_maybe_small_cave_to_repeat(None)
    };
    Ok(paths)
}

fn generate_reverse_sub_paths<'a>(
    connections: &'a HashMap<Point, HashSet<Point>>,
    from: &'a Point,
    maybe_small_cave_to_repeat: Option<&Point>,
    mut visited_small: HashMap<&'a Point, usize>,
) -> Vec<Vec<&'a Point>> {
    if let Point::SmallCave(_) = from {
        *visited_small.entry(from).or_insert(0) += 1;
    }
    let maybe_next_points = connections.get(from);
    if let Some(next_points) = maybe_next_points {
        let next_points_unvisited_so_far = next_points.iter().filter(|p| {
            let max_visit_threshold: usize =
                if let Some(small_point_to_repeat) = maybe_small_cave_to_repeat {
                    if small_point_to_repeat == *p {
                        THRESHOLD_FOR_REPEATABLE
                    } else {
                        1
                    }
                } else {
                    1
                };
            visited_small
                .get(*p)
                .map(|visited_count| *visited_count < max_visit_threshold)
                .unwrap_or(true)
        });
        let next_pathss =
            next_points_unvisited_so_far
                .into_iter()
                .map(|next_point| match *next_point {
                    End => vec![vec![&End]],
                    Start => vec![],
                    _ => generate_reverse_sub_paths(
                        connections,
                        next_point,
                        maybe_small_cave_to_repeat,
                        visited_small.clone(),
                    ),
                });
        next_pathss
            .flat_map(|mut paths| {
                paths.iter_mut().for_each(|path| path.push(from));
                paths
            })
            .filter(|path| path.starts_with(&[&End]))
            .collect()
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use super::*;

    static TEST_INPUT: &str = "start-A
start-b
A-c
A-b
b-d
A-end
b-end";

    static TEST_INPUT_2: &str = "dc-end
HN-start
start-kj
dc-start
dc-HN
LN-dc
HN-end
kj-sa
kj-HN
kj-dc";

    static TEST_INPUT_3: &str = "fs-end
he-DX
fs-he
start-DX
pj-DX
end-zg
zg-sl
zg-pj
pj-he
RW-he
fs-DX
pj-RW
zg-RW
start-pj
he-WI
zg-he
pj-fs
start-RW";

    #[test]
    fn parse_test() {
        let i = parse(TEST_INPUT).unwrap();
        assert_eq!(
            Input(HashMap::from_iter([
                (
                    BigCave("A".to_string()),
                    HashSet::from_iter([
                        Start,
                        SmallCave("c".to_string()),
                        SmallCave("b".to_string()),
                        End
                    ])
                ),
                (
                    SmallCave("b".to_string()),
                    HashSet::from_iter([
                        Start,
                        End,
                        BigCave("A".to_string()),
                        SmallCave("d".to_string())
                    ])
                ),
                (
                    SmallCave("c".to_string()),
                    HashSet::from_iter([BigCave("A".to_string())])
                ),
                (
                    SmallCave("d".to_string()),
                    HashSet::from_iter([SmallCave("b".to_string())])
                ),
                (
                    Start,
                    HashSet::from_iter([BigCave("A".to_string()), SmallCave("b".to_string())])
                ),
                (
                    End,
                    HashSet::from_iter([BigCave("A".to_string()), SmallCave("b".to_string())])
                )
            ])),
            i
        );
    }

    #[test]
    fn generate_paths_no_small_cave_repeats_test() {
        let i = parse(TEST_INPUT).unwrap();
        let r = generate_paths(&i, false).unwrap();
        assert_eq!(10, r.len())
    }

    #[test]
    fn generate_paths_no_small_cave_repeats_2_test() {
        let i = parse(TEST_INPUT_2).unwrap();
        let r = generate_paths(&i, false).unwrap();
        assert_eq!(19, r.len())
    }

    #[test]
    fn generate_paths_no_small_cave_repeats_3_test() {
        let i = parse(TEST_INPUT_3).unwrap();
        let r = generate_paths(&i, false).unwrap();
        assert_eq!(226, r.len())
    }

    #[test]
    fn generate_paths_with_small_cave_repeats_test() {
        let i = parse(TEST_INPUT).unwrap();
        let r = generate_paths(&i, true).unwrap();
        assert_eq!(36, r.len())
    }

    #[test]
    fn generate_paths_with_small_cave_repeats_2_test() {
        let i = parse(TEST_INPUT_2).unwrap();
        let r = generate_paths(&i, true).unwrap();
        assert_eq!(103, r.len())
    }

    #[test]
    fn generate_paths_with_small_cave_repeats_3_test() {
        let i = parse(TEST_INPUT_3).unwrap();
        let r = generate_paths(&i, true).unwrap();
        assert_eq!(3509, r.len())
    }
}
