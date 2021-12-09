use std::result::Result as StdResult;

use anyhow::Result;
use combine::easy::{Error, Info};
use combine::parser::char::*;
use combine::*;
use itertools::Itertools;
use std::collections::HashSet;

const INPUT: &str = include_str!("../data/day_09_input");

const BASIN_HEIGHT_LIMIT: usize = 9;

pub fn run() -> Result<()> {
    println!("*** Day 9: Smoke Basin ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let sol_1 = part_1_sol(&input);
    println!("Solution 1: {:?}", sol_1);
    let sol_2 = part_2_sol(&input);
    println!("Solution 2: {:?}", sol_2);
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Input {
    rows: Vec<Vec<usize>>,
    max_coords: Coords,
}

fn part_1_sol(i: &Input) -> usize {
    let local_lows = i.find_local_lows();
    local_lows
        .iter()
        .fold(0, |acc, coords| acc + i.rows[coords.row][coords.col] + 1)
}

fn part_2_sol(i: &Input) -> Option<usize> {
    let basin_coords = i.find_basins();
    basin_coords
        .iter()
        .map(|coords| coords.len())
        .sorted_by_key(|u| *u)
        .rev()
        .take(3)
        .reduce(|a, b| a * b)
}

impl Input {
    #![allow(unused_mut)]
    fn find_local_lows(&self) -> Vec<Coords> {
        self.rows
            .iter()
            .enumerate()
            .fold(vec![], |mut acc, (row_idx, row)| {
                row.iter()
                    .enumerate()
                    .fold(acc, |mut inner_acc, (col_idx, _)| {
                        let coords = Coords {
                            row: row_idx,
                            col: col_idx,
                        };
                        if self.is_local_low(&coords) {
                            inner_acc.push(coords)
                        }
                        inner_acc
                    })
            })
    }

    fn find_basins(&self) -> Vec<HashSet<Coords>> {
        let local_lows = self.find_local_lows();
        local_lows
            .iter()
            .map(|local_low| self.find_basin_around(local_low, HashSet::new()))
            .collect()
    }

    fn find_basin_around(&self, point: &Coords, current_basin: HashSet<Coords>) -> HashSet<Coords> {
        let mut adjacents = self.adjacents(point);
        adjacents.retain(|coords| !current_basin.contains(coords));
        adjacents
            .into_iter()
            .fold(current_basin, |mut acc, coords| {
                if self.rows[coords.row][coords.col] != BASIN_HEIGHT_LIMIT {
                    if !acc.contains(&coords) {
                        acc.insert(coords);
                    }
                    self.find_basin_around(&coords, acc)
                } else {
                    acc
                }
            })
    }

    fn is_local_low(&self, coords: &Coords) -> bool {
        let coord_value = self.rows[coords.row][coords.col];
        self.adjacents(coords).iter().all(|adjacent_coords| {
            self.rows[adjacent_coords.row][adjacent_coords.col] > coord_value
        })
    }

    #[allow(clippy::collapsible_else_if)]
    fn adjacents(&self, Coords { row, col }: &Coords) -> Vec<Coords> {
        if *row == 0 {
            // top row
            if *col == 0 {
                // top left corner
                vec![Coords { row: 0, col: 1 }, Coords { row: 1, col: 0 }]
            } else if *col == self.max_coords.col {
                // top right corner
                vec![
                    Coords {
                        row: 0,
                        col: col - 1,
                    },
                    Coords { row: 1, col: *col },
                ]
            } else {
                // everything inside
                vec![
                    Coords {
                        row: 0,
                        col: col - 1,
                    },
                    Coords {
                        row: 0,
                        col: col + 1,
                    },
                    Coords { row: 1, col: *col },
                ]
            }
        } else if *row == self.max_coords.row {
            // bottom row
            if *col == 0 {
                // bottom left corner
                vec![
                    Coords { row: *row, col: 1 },
                    Coords {
                        row: row - 1,
                        col: 0,
                    },
                ]
            } else if *col == self.max_coords.col {
                // bottom right corner
                vec![
                    Coords {
                        row: *row,
                        col: col - 1,
                    },
                    Coords {
                        row: row - 1,
                        col: *col,
                    },
                ]
            } else {
                // everything inside
                vec![
                    Coords {
                        row: *row,
                        col: col - 1,
                    },
                    Coords {
                        row: *row,
                        col: col + 1,
                    },
                    Coords {
                        row: row - 1,
                        col: *col,
                    },
                ]
            }
        } else {
            if *col == 0 {
                vec![
                    Coords {
                        row: row - 1,
                        col: *col,
                    },
                    Coords {
                        row: row + 1,
                        col: *col,
                    },
                    Coords {
                        row: *row,
                        col: col + 1,
                    },
                ]
            } else if *col == self.max_coords.col {
                vec![
                    Coords {
                        row: row - 1,
                        col: *col,
                    },
                    Coords {
                        row: row + 1,
                        col: *col,
                    },
                    Coords {
                        row: *row,
                        col: col - 1,
                    },
                ]
            } else if *row < self.max_coords.row && *col < self.max_coords.col {
                vec![
                    Coords {
                        row: row - 1,
                        col: *col,
                    },
                    Coords {
                        row: row + 1,
                        col: *col,
                    },
                    Coords {
                        row: *row,
                        col: col + 1,
                    },
                    Coords {
                        row: *row,
                        col: col - 1,
                    },
                ]
            } else {
                // Prevents returning adjacents for things that are out of bounds...
                vec![]
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Coords {
    row: usize,
    col: usize,
}

fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
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
                "Not all rows have the same length",
            )))
        }
    });
    let (r, _) = parser.easy_parse(s)?;
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &str = "2199943210
3987894921
9856789892
8767896789
9899965678";

    #[test]
    fn parse_test() {
        let i = parse(TEST_INPUT).unwrap();
        assert_eq!(
            Input {
                rows: vec![
                    vec![2, 1, 9, 9, 9, 4, 3, 2, 1, 0],
                    vec![3, 9, 8, 7, 8, 9, 4, 9, 2, 1],
                    vec![9, 8, 5, 6, 7, 8, 9, 8, 9, 2],
                    vec![8, 7, 6, 7, 8, 9, 6, 7, 8, 9],
                    vec![9, 8, 9, 9, 9, 6, 5, 6, 7, 8],
                ],
                max_coords: Coords { row: 4, col: 9 },
            },
            i
        );
    }

    #[test]
    fn part_1_sol_test() {
        let i = parse(TEST_INPUT).unwrap();
        let s = part_1_sol(&i);
        assert_eq!(15, s)
    }

    #[test]
    fn part_2_sol_test() {
        let i = parse(TEST_INPUT).unwrap();
        let s = part_2_sol(&i).unwrap();
        assert_eq!(1134, s)
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
                Coords { row: 1, col: 2 }
            ],
            r2
        );
        let r3 = i.adjacents(&Coords { row: 0, col: 9 });
        assert_eq!(
            vec![Coords { row: 0, col: 8 }, Coords { row: 1, col: 9 }],
            r3
        );
        // bottom row
        let r4 = i.adjacents(&Coords { row: 4, col: 0 });
        assert_eq!(
            vec![Coords { row: 4, col: 1 }, Coords { row: 3, col: 0 }],
            r4
        );
        let r5 = i.adjacents(&Coords { row: 4, col: 6 });
        assert_eq!(
            vec![
                Coords { row: 4, col: 5 },
                Coords { row: 4, col: 7 },
                Coords { row: 3, col: 6 }
            ],
            r5
        );
        let r6 = i.adjacents(&Coords { row: 4, col: 9 });
        assert_eq!(
            vec![Coords { row: 4, col: 8 }, Coords { row: 3, col: 9 }],
            r6
        );
        // middle row
        let r7 = i.adjacents(&Coords { row: 3, col: 0 });
        assert_eq!(
            vec![
                Coords { row: 2, col: 0 },
                Coords { row: 4, col: 0 },
                Coords { row: 3, col: 1 }
            ],
            r7
        );
        let r8 = i.adjacents(&Coords { row: 1, col: 3 });
        assert_eq!(
            vec![
                Coords { row: 0, col: 3 },
                Coords { row: 2, col: 3 },
                Coords { row: 1, col: 4 },
                Coords { row: 1, col: 2 },
            ],
            r8
        );
        let r9 = i.adjacents(&Coords { row: 2, col: 9 });
        assert_eq!(
            vec![
                Coords { row: 1, col: 9 },
                Coords { row: 3, col: 9 },
                Coords { row: 2, col: 8 }
            ],
            r9
        );
    }
}
