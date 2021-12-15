use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::result::Result as StdResult;

use anyhow::Result;
use combine::easy::{Error, Info};
use combine::parser::char::*;
use combine::*;

pub const INPUT: &str = include_str!("../data/day_15_input");

const UNEXPLORED_RISK: usize = usize::MAX;

pub fn run() -> Result<()> {
    println!("*** Day 15: Chiton ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let sol_1 = input.lowest_risk_to_end();
    println!("Solution 1: {:?}", sol_1);

    let expanded_input = input.expand(5);
    let sol_2 = expanded_input.lowest_risk_to_end();
    println!("Solution 2: {:?}", sol_2);
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub struct Input {
    rows: Vec<Vec<usize>>,
    max_coords: Coords,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct RiskToPosition {
    risk: usize,
    coords: Coords,
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for RiskToPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice that the we flip the ordering on risk.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .risk
            .cmp(&self.risk)
            .then_with(|| self.coords.cmp(&other.coords))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for RiskToPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Input {
    pub fn lowest_risk_to_end(&self) -> Option<usize> {
        // Read the docs for BinaryHeap :p
        let start = Coords { row: 0, col: 0 };
        let end = self.max_coords;

        // distances_from_start[coords] = current shortest distance from `start` to `coords`
        let mut distances_from_start: HashMap<Coords, usize> =
            HashMap::with_capacity((self.max_coords.row + 1) * (self.max_coords.col + 1));

        let mut to_visit_prioritised_q = BinaryHeap::new();

        // We're at `start`, with a zero risk
        distances_from_start.insert(start, 0);
        to_visit_prioritised_q.push(RiskToPosition {
            risk: 0,
            coords: start,
        });

        // Examine the frontier with lower risk positions first
        while let Some(RiskToPosition { risk, coords }) = to_visit_prioritised_q.pop() {
            // Done; just exit
            if coords == end {
                return Some(risk);
            } else {
                let known_lowest_risk_to_current = distances_from_start
                    .get(&coords)
                    .copied()
                    .unwrap_or(UNEXPLORED_RISK);
                if risk > known_lowest_risk_to_current {
                    // Skip exploring this path; we already know of a less risky way of getting here
                    continue;
                } else {
                    // For each coords we can reach, see if we can find a way with
                    // a lower risk going through some new coords
                    for adjacent in self.adjacents(&coords) {
                        let current_known_lowest_risk_to_adjacent = distances_from_start
                            .entry(adjacent)
                            .or_insert(UNEXPLORED_RISK);
                        let next_tally = RiskToPosition {
                            risk: risk + self.rows[adjacent.row][adjacent.col],
                            coords: adjacent,
                        };
                        // If so, add it to the frontier and continue
                        if next_tally.risk < *current_known_lowest_risk_to_adjacent {
                            to_visit_prioritised_q.push(next_tally);
                            *current_known_lowest_risk_to_adjacent = next_tally.risk;
                        }
                    }
                }
            }
        }

        // End not reachable
        None
    }

    pub fn expand(&self, expand_factor: usize) -> Input {
        let expanded_rows: Vec<_> = (0..expand_factor)
            .flat_map(|row_copy_idx| {
                let expanded_rows: Vec<_> = self
                    .rows
                    .iter()
                    .map(|row| {
                        (0..expand_factor)
                            .flat_map(|col_copy_idx| {
                                row.iter()
                                    .map(|v| {
                                        let next_value = v + col_copy_idx + row_copy_idx;
                                        if next_value > 9 {
                                            next_value % 9
                                        } else {
                                            next_value
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .collect::<Vec<usize>>()
                    })
                    .collect();
                expanded_rows
            })
            .collect();
        let expanded_max_coords = Coords {
            row: (self.max_coords.row + 1) * expand_factor - 1,
            col: (self.max_coords.col + 1) * expand_factor - 1,
        };
        Input {
            rows: expanded_rows,
            max_coords: expanded_max_coords,
        }
    }

    fn adjacents(&self, Coords { row, col }: &Coords) -> Vec<Coords> {
        let row_i = *row as isize;
        let col_i = *col as isize;
        if *row <= self.max_coords.row && *col <= self.max_coords.col {
            vec![
                (row_i - 1, col_i), // N
                (row_i, col_i - 1), // W
                (row_i, col_i + 1), // E
                (row_i + 1, col_i), // S
            ]
            .into_iter()
            .filter(|(r, c)| {
                !(*r < 0
                    || *c < 0
                    || *r > self.max_coords.row as isize
                    || *c > self.max_coords.col as isize)
            })
            .map(|(row, col)| Coords {
                row: row as usize,
                col: col as usize,
            })
            .collect()
        } else {
            vec![]
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
pub struct Coords {
    row: usize,
    col: usize,
}

pub fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
    let line_parser = many1(digit().and_then(|n_s| n_s.to_string().parse::<usize>()));
    let mut parser = many1(line_parser.skip(spaces())).and_then(|rows: Vec<Vec<usize>>| {
        let first_row_length = rows[0].len(); // our parser ensures at least 1
        let all_rows_same_length = rows.iter().all(|row| row.len() == first_row_length);
        if all_rows_same_length {
            let max_coords = Coords {
                row: rows.len() - 1,
                col: first_row_length - 1,
            };
            Ok(Input { rows, max_coords })
        } else {
            Err(Error::Unexpected(Info::Static(
                "Not all rows have the same number of columns",
            )))
        }
    });
    let (r, _) = parser.easy_parse(s)?;
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &str = "1163751742
1381373672
2136511328
3694931569
7463417111
1319128137
1359912421
3125421639
1293138521
2311944581";

    #[test]
    fn sol_1_test() {
        let i = parse(TEST_INPUT).unwrap();
        let r = i.lowest_risk_to_end();
        assert_eq!(Some(40), r);
    }

    #[test]
    fn sol_2_test() {
        let i = parse(TEST_INPUT).unwrap();
        let expanded = i.expand(5);
        let r = expanded.lowest_risk_to_end();
        assert_eq!(Some(315), r);
    }

    #[test]
    fn parse_test() {
        let i = parse(TEST_INPUT).unwrap();
        assert_eq!(
            Input {
                rows: vec![
                    vec![1, 1, 6, 3, 7, 5, 1, 7, 4, 2],
                    vec![1, 3, 8, 1, 3, 7, 3, 6, 7, 2],
                    vec![2, 1, 3, 6, 5, 1, 1, 3, 2, 8],
                    vec![3, 6, 9, 4, 9, 3, 1, 5, 6, 9],
                    vec![7, 4, 6, 3, 4, 1, 7, 1, 1, 1],
                    vec![1, 3, 1, 9, 1, 2, 8, 1, 3, 7],
                    vec![1, 3, 5, 9, 9, 1, 2, 4, 2, 1],
                    vec![3, 1, 2, 5, 4, 2, 1, 6, 3, 9],
                    vec![1, 2, 9, 3, 1, 3, 8, 5, 2, 1],
                    vec![2, 3, 1, 1, 9, 4, 4, 5, 8, 1],
                ],
                max_coords: Coords { row: 9, col: 9 },
            },
            i
        )
    }

    #[test]
    fn adjacent_points_test() {
        let i = parse(TEST_INPUT).unwrap();
        // top row
        let r1 = i.adjacents(&Coords { row: 0, col: 0 });
        assert_eq!(
            vec![Coords { row: 0, col: 1 }, Coords { row: 1, col: 0 }],
            r1
        );
        let r2 = i.adjacents(&Coords { row: 0, col: 2 });
        assert_eq!(
            vec![
                Coords { row: 0, col: 1 },
                Coords { row: 0, col: 3 },
                Coords { row: 1, col: 2 },
            ],
            r2
        );
        let r3 = i.adjacents(&Coords { row: 0, col: 9 });
        assert_eq!(
            vec![Coords { row: 0, col: 8 }, Coords { row: 1, col: 9 }],
            r3
        );
        // bottom row
        let r4 = i.adjacents(&Coords { row: 9, col: 0 });
        assert_eq!(
            vec![Coords { row: 8, col: 0 }, Coords { row: 9, col: 1 }],
            r4
        );
        let r5 = i.adjacents(&Coords { row: 9, col: 5 });
        assert_eq!(
            vec![
                Coords { row: 8, col: 5 },
                Coords { row: 9, col: 4 },
                Coords { row: 9, col: 6 },
            ],
            r5
        );
        let r6 = i.adjacents(&Coords { row: 9, col: 9 });
        assert_eq!(
            vec![Coords { row: 8, col: 9 }, Coords { row: 9, col: 8 }],
            r6
        );
        // middle row
        let r7 = i.adjacents(&Coords { row: 3, col: 0 });
        assert_eq!(
            vec![
                Coords { row: 2, col: 0 },
                Coords { row: 3, col: 1 },
                Coords { row: 4, col: 0 },
            ],
            r7
        );
        let r8 = i.adjacents(&Coords { row: 1, col: 3 });
        assert_eq!(
            vec![
                Coords { row: 0, col: 3 },
                Coords { row: 1, col: 2 },
                Coords { row: 1, col: 4 },
                Coords { row: 2, col: 3 },
            ],
            r8
        );
        let r9 = i.adjacents(&Coords { row: 2, col: 9 });
        assert_eq!(
            vec![
                Coords { row: 1, col: 9 },
                Coords { row: 2, col: 8 },
                Coords { row: 3, col: 9 },
            ],
            r9
        );
        // out of bounds
        let r10 = i.adjacents(&Coords { row: 10, col: 10 });
        assert!(r10.is_empty());
    }
}
