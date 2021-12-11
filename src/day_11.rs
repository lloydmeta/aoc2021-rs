use std::result::Result as StdResult;

use anyhow::Result;
use combine::easy::{Error, Info};
use combine::parser::char::*;
use combine::*;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

const FLASH_POINT: usize = 9;
const START_POINT: usize = 0;

const INPUT: &str = include_str!("../data/day_11_input");

pub fn run() -> Result<()> {
    println!("*** Day 11: Dumbo Octopus ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let sol_1 = sol_1(&input);
    println!("Solution 1: {:?}", sol_1);
    let sol_2 = sol_2(&input);
    println!("Solution 2: {:?}", sol_2);
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Input {
    rows: Vec<Vec<usize>>,
    max_coords: Coords,
    flashes: usize,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Coords {
    row: usize,
    col: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct Step {
    day: usize,
    rows: Vec<Vec<usize>>,
    flashes: usize,
}

impl Display for Step {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Day: {}", self.day)?;
        writeln!(f, "Flashes: {}", self.flashes)?;
        for (row_idx, row) in self.rows.iter().enumerate() {
            for v in row {
                write!(f, "{}", v)?;
            }
            if !row_idx >= self.rows.len() - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

fn sol_1(i: &Input) -> usize {
    i.simulate(100).fold(0, |acc, step| step.flashes + acc)
}

fn sol_2(i: &Input) -> Option<usize> {
    let octo_count = (i.max_coords.row + 1) * (i.max_coords.col + 1);
    i.simulate(usize::MAX)
        .find(|step| step.flashes == octo_count)
        .map(|s| s.day)
}

impl Input {
    fn adjacents(&self, Coords { row, col }: &Coords) -> Vec<Coords> {
        let row_i = *row as isize;
        let col_i = *col as isize;
        if *row <= self.max_coords.row && *col <= self.max_coords.col {
            vec![
                (row_i - 1, col_i - 1), // NW
                (row_i - 1, col_i),     // N
                (row_i - 1, col_i + 1), // NE
                (row_i, col_i - 1),     // W
                (row_i, col_i + 1),     // E
                (row_i + 1, col_i - 1), // SW
                (row_i + 1, col_i),     // S
                (row_i + 1, col_i + 1), // SE
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

    fn simulate(&self, days: usize) -> impl Iterator<Item = Step> + '_ {
        let init_flashes = self.flashes;
        let init_step = self.rows.clone();
        (0..=days)
            .into_iter()
            .scan(init_step, move |rows_state, step_idx| {
                if step_idx == 0 {
                    Some(Step {
                        day: step_idx,
                        rows: rows_state.clone(),
                        flashes: init_flashes,
                    })
                } else {
                    rows_state
                        .iter_mut()
                        .for_each(|row| row.iter_mut().map(|v| *v += 1).collect());
                    let mut flashed_coords_in_step = HashSet::new();
                    loop {
                        let mut new_row_state = rows_state.clone();
                        let mut flashed_coords_current_flash_round = HashSet::new();
                        for (row_idx, row) in new_row_state.iter_mut().enumerate() {
                            for (col_idx, current_val) in row.iter_mut().enumerate() {
                                let coords = Coords {
                                    row: row_idx,
                                    col: col_idx,
                                };
                                // if it's above the flash point anyway, then just insert it into the current
                                // round of flash coordinates; we'll set this to 0 at the end of the step anyways
                                if *current_val > FLASH_POINT
                                    || flashed_coords_in_step.contains(&coords)
                                {
                                    if !flashed_coords_in_step.contains(&coords) {
                                        flashed_coords_current_flash_round.insert(coords);
                                    }
                                } else {
                                    let adjacent_coords = self.adjacents(&coords);
                                    let adjacent_values_that_have_not_flashed_during_this_step =
                                        adjacent_coords
                                            .iter()
                                            // ignore ones that have already flashed once during the step
                                            .filter(|coords| {
                                                !flashed_coords_in_step.contains(*coords)
                                            })
                                            .filter_map(|Coords { row, col }| {
                                                rows_state.get(*row).and_then(|r| r.get(*col))
                                            });
                                    let adjacent_values_that_will_flash =
                                        adjacent_values_that_have_not_flashed_during_this_step
                                            .filter(|v| **v > FLASH_POINT)
                                            .count();
                                    *current_val += adjacent_values_that_will_flash;
                                }
                            }
                        }

                        for Coords { row, col } in flashed_coords_current_flash_round.iter() {
                            new_row_state[*row][*col] = START_POINT;
                            flashed_coords_in_step.insert(Coords {
                                row: *row,
                                col: *col,
                            });
                        }
                        *rows_state = new_row_state.clone();
                        if flashed_coords_current_flash_round.is_empty() {
                            break;
                        }
                    }
                    Some(Step {
                        day: step_idx,
                        rows: rows_state.clone(),
                        flashes: flashed_coords_in_step.len(),
                    })
                }
            })
    }
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
            let flashes = rows.iter().fold(0, |acc, row| {
                row.iter().filter(|v| **v == FLASH_POINT).count() + acc
            });
            Ok(Input {
                rows,
                max_coords,
                flashes,
            })
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

    static TEST_INPUT: &str = "5483143223
2745854711
5264556173
6141336146
6357385478
4167524645
2176841721
6882881134
4846848554
5283751526";

    #[test]
    fn parse_test() {
        let i = parse(TEST_INPUT).unwrap();
        assert_eq!(
            Input {
                rows: vec![
                    vec![5, 4, 8, 3, 1, 4, 3, 2, 2, 3],
                    vec![2, 7, 4, 5, 8, 5, 4, 7, 1, 1],
                    vec![5, 2, 6, 4, 5, 5, 6, 1, 7, 3],
                    vec![6, 1, 4, 1, 3, 3, 6, 1, 4, 6],
                    vec![6, 3, 5, 7, 3, 8, 5, 4, 7, 8],
                    vec![4, 1, 6, 7, 5, 2, 4, 6, 4, 5],
                    vec![2, 1, 7, 6, 8, 4, 1, 7, 2, 1],
                    vec![6, 8, 8, 2, 8, 8, 1, 1, 3, 4],
                    vec![4, 8, 4, 6, 8, 4, 8, 5, 5, 4],
                    vec![5, 2, 8, 3, 7, 5, 1, 5, 2, 6],
                ],
                max_coords: Coords { row: 9, col: 9 },
                flashes: 0
            },
            i
        );
    }

    #[test]
    fn simulate_test() {
        let i = parse(TEST_INPUT).unwrap();
        let simulated_steps: Vec<_> = i.simulate(10).collect();
        assert_eq!(0, simulated_steps[0].day);
        assert_eq!(0, simulated_steps[0].flashes);
        assert_eq!(1, simulated_steps[1].day);
        assert_eq!(0, simulated_steps[1].flashes);
        assert_eq!(2, simulated_steps[2].day);
        assert_eq!(35, simulated_steps[2].flashes);
        assert_eq!(3, simulated_steps[3].day);
        assert_eq!(45, simulated_steps[3].flashes);
        assert_eq!(10, simulated_steps[10].day);
        assert_eq!(29, simulated_steps[10].flashes);
    }

    #[test]
    fn sol_1_test() {
        let i = parse(TEST_INPUT).unwrap();
        let r = sol_1(&i);
        assert_eq!(1656, r);
    }

    #[test]
    fn sol_2_test() {
        let i = parse(TEST_INPUT).unwrap();
        let r = sol_2(&i);
        assert_eq!(Some(195), r);
    }

    #[test]
    fn adjacent_points_test() {
        let i = parse(TEST_INPUT).unwrap();
        // top row
        let r1 = i.adjacents(&Coords { row: 0, col: 0 });
        assert_eq!(
            vec![
                Coords { row: 0, col: 1 },
                Coords { row: 1, col: 0 },
                Coords { row: 1, col: 1 }
            ],
            r1
        );
        let r2 = i.adjacents(&Coords { row: 0, col: 2 });
        assert_eq!(
            vec![
                Coords { row: 0, col: 1 },
                Coords { row: 0, col: 3 },
                Coords { row: 1, col: 1 },
                Coords { row: 1, col: 2 },
                Coords { row: 1, col: 3 },
            ],
            r2
        );
        let r3 = i.adjacents(&Coords { row: 0, col: 9 });
        assert_eq!(
            vec![
                Coords { row: 0, col: 8 },
                Coords { row: 1, col: 8 },
                Coords { row: 1, col: 9 }
            ],
            r3
        );
        // bottom row
        let r4 = i.adjacents(&Coords { row: 9, col: 0 });
        assert_eq!(
            vec![
                Coords { row: 8, col: 0 },
                Coords { row: 8, col: 1 },
                Coords { row: 9, col: 1 }
            ],
            r4
        );
        let r5 = i.adjacents(&Coords { row: 9, col: 5 });
        assert_eq!(
            vec![
                Coords { row: 8, col: 4 },
                Coords { row: 8, col: 5 },
                Coords { row: 8, col: 6 },
                Coords { row: 9, col: 4 },
                Coords { row: 9, col: 6 }
            ],
            r5
        );
        let r6 = i.adjacents(&Coords { row: 9, col: 9 });
        assert_eq!(
            vec![
                Coords { row: 8, col: 8 },
                Coords { row: 8, col: 9 },
                Coords { row: 9, col: 8 }
            ],
            r6
        );
        // middle row
        let r7 = i.adjacents(&Coords { row: 3, col: 0 });
        assert_eq!(
            vec![
                Coords { row: 2, col: 0 },
                Coords { row: 2, col: 1 },
                Coords { row: 3, col: 1 },
                Coords { row: 4, col: 0 },
                Coords { row: 4, col: 1 }
            ],
            r7
        );
        let r8 = i.adjacents(&Coords { row: 1, col: 3 });
        assert_eq!(
            vec![
                Coords { row: 0, col: 2 },
                Coords { row: 0, col: 3 },
                Coords { row: 0, col: 4 },
                Coords { row: 1, col: 2 },
                Coords { row: 1, col: 4 },
                Coords { row: 2, col: 2 },
                Coords { row: 2, col: 3 },
                Coords { row: 2, col: 4 }
            ],
            r8
        );
        let r9 = i.adjacents(&Coords { row: 2, col: 9 });
        assert_eq!(
            vec![
                Coords { row: 1, col: 8 },
                Coords { row: 1, col: 9 },
                Coords { row: 2, col: 8 },
                Coords { row: 3, col: 8 },
                Coords { row: 3, col: 9 },
            ],
            r9
        );
        // out of bounds
        let r10 = i.adjacents(&Coords { row: 10, col: 10 });
        assert!(r10.is_empty());
    }
}
