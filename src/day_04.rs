use std::collections::{HashMap, HashSet};

use anyhow::Result;
use itertools::{FoldWhile, Itertools};

const BINGO_COUNT: u8 = 5;
const INPUT: &str = include_str!("../data/day_04_input");

pub fn run() -> Result<()> {
    println!("*** Day 4: Giant Squid ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let solutions = winning_solutions(&input);
    println!("Solution 1: {:?}\n", solutions.sol_1());
    println!("Solution 2: {:?}\n", solutions.sol_2());
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Bingo {
    numbers_to_draw: NumbersToDraw,
    boards: Vec<Board>,
}

#[derive(Debug, Eq, PartialEq)]
struct NumbersToDraw(Vec<usize>);

#[derive(Debug, Eq, PartialEq)]
struct Board {
    numbers: Vec<Vec<usize>>,
}

// Row-to-cols with hits
#[derive(Debug, Eq, PartialEq, Clone)]
struct Hits(HashMap<usize, HashSet<usize>>);

#[derive(Debug, Eq, PartialEq)]
struct BoardIdxToHits(Vec<Hits>);

struct Solutions(Vec<usize>);

impl Solutions {
    fn sol_1(&self) -> Option<usize> {
        self.0.first().cloned()
    }
    fn sol_2(&self) -> Option<usize> {
        self.0.last().cloned()
    }
}

fn parse(s: &str) -> Result<Bingo> {
    let by_section: Vec<_> = s.split("\n\n").collect();
    let numbers_to_draw = if let Some(numbers_sec) = by_section.get(0) {
        let numbers: Vec<usize> = numbers_sec
            .split(',')
            .filter_map(|n| n.parse().ok())
            .collect();
        NumbersToDraw(numbers)
    } else {
        bail!("Failed to parse numbers to draw")
    };

    let boards: Vec<Vec<Vec<usize>>> = by_section
        .iter()
        .skip(1)
        .map(|board_s| {
            let numbers = board_s
                .lines()
                .map(|row| {
                    let row_trimmed = row.trim();
                    let numbers: Vec<usize> = row_trimmed
                        .split(' ')
                        .filter_map(|num_s| num_s.parse().ok())
                        .collect();
                    numbers
                })
                .collect();
            numbers
        })
        .collect();

    if let Some(first_board) = boards.get(0) {
        let rows_count = first_board.len();
        if let Some(first_row) = first_board.get(0) {
            let cols_count = first_row.len();
            if boards.iter().all(|board| {
                board.len() == rows_count
                    && board.get(0).map(|row| row.len()).unwrap_or(0) == cols_count
            }) {
                let boards = boards
                    .into_iter()
                    .map(|board| Board { numbers: board })
                    .collect();
                Ok(Bingo {
                    numbers_to_draw,
                    boards,
                })
            } else {
                bail!("Non-rectangular input")
            }
        } else {
            bail!("No columns parsed")
        }
    } else {
        bail!("No boards parsed")
    }
}

fn winning_solutions(b: &Bingo) -> Solutions {
    let boards_count = b.boards.len();
    let (_, _, winning_solutions) = b
        .numbers_to_draw
        .0
        .iter()
        .fold_while(
            (
                BoardIdxToHits(vec![
                    Hits(HashMap::with_capacity(BINGO_COUNT as usize));
                    boards_count
                ]),
                HashSet::with_capacity(boards_count),
                Vec::with_capacity(boards_count),
            ),
            |(mut board_idx_to_hits, mut winning_board_idxs, mut winning_solutions),
             drawn_number| {
                for (board_idx, Board { numbers }) in b.boards.iter().enumerate() {
                    if !winning_board_idxs.contains(&board_idx) {
                        for (row_idx, row) in numbers.iter().enumerate() {
                            for (col_idx, num) in row.iter().enumerate() {
                                if num == drawn_number {
                                    // println!("Hit on board idx [{}] at ({}, {})", board_idx, row_idx, col_idx);
                                    board_idx_to_hits.0[board_idx]
                                        .0
                                        .entry(row_idx)
                                        .or_insert_with(|| {
                                            HashSet::with_capacity(BINGO_COUNT as usize)
                                        })
                                        .insert(col_idx);
                                }
                            }
                        }
                    }
                }
                board_idx_to_hits
                    .0
                    .iter()
                    .enumerate()
                    .for_each(|(board_idx, hits)| {
                        if !winning_board_idxs.contains(&board_idx) && has_bingo(hits) {
                            let solution = {
                                // safe idx gets
                                let hits_for_board = &board_idx_to_hits.0[board_idx];
                                let sum_non_drawn = &b.boards[board_idx]
                                    .numbers
                                    .iter()
                                    .enumerate()
                                    .fold(0, |acc, (row_idx, row)| {
                                        row.iter().enumerate().fold(acc, |acc, (col_idx, num)| {
                                            if hits_for_board
                                                .0
                                                .get(&row_idx)
                                                .map(|col_hits| col_hits.iter().contains(&col_idx))
                                                .unwrap_or(false)
                                            {
                                                acc
                                            } else {
                                                acc + num
                                            }
                                        })
                                    });
                                sum_non_drawn * drawn_number
                            };
                            winning_board_idxs.insert(board_idx);
                            winning_solutions.push(solution)
                        }
                    });
                if winning_board_idxs.len() == b.boards.len() {
                    FoldWhile::Done((board_idx_to_hits, winning_board_idxs, winning_solutions))
                } else {
                    FoldWhile::Continue((board_idx_to_hits, winning_board_idxs, winning_solutions))
                }
            },
        )
        .into_inner();
    Solutions(winning_solutions)
}

fn has_bingo(Hits(hit_ats): &Hits) -> bool {
    // Can probably optimise this into two numbers but meh
    let (row_idx_to_hit_count, col_idx_to_hit_count) = hit_ats.iter().fold(
        (
            HashMap::with_capacity(hit_ats.len()),
            HashMap::with_capacity(hit_ats.len()),
        ),
        |(mut row_idx_to_hit_count, mut col_idx_to_hit_count), (row, col_hits)| {
            *row_idx_to_hit_count.entry(row).or_insert(0) += col_hits.len() as u8;
            for column in col_hits {
                *col_idx_to_hit_count.entry(column).or_insert(0) += 1u8;
            }
            (row_idx_to_hit_count, col_idx_to_hit_count)
        },
    );
    row_idx_to_hit_count
        .iter()
        .any(|(_, hit_counts)| *hit_counts >= BINGO_COUNT)
        || col_idx_to_hit_count
            .iter()
            .any(|(_, hit_counts)| *hit_counts >= BINGO_COUNT)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &str =
        "7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1

22 13 17 11  0
 8  2 23  4 24
21  9 14 16  7
 6 10  3 18  5
 1 12 20 15 19

 3 15  0  2 22
 9 18 13 17  5
19  8  7 25 23
20 11 10 24  4
14 21 16 12  6

14 21 17 24  4
10 16 15  9 19
18  8 23 26 20
22 11 13  6  5
 2  0 12  3  7";

    #[test]
    fn parse_test() {
        let r = parse(TEST_INPUT).unwrap();
        let expected = Bingo {
            numbers_to_draw: NumbersToDraw(vec![
                7, 4, 9, 5, 11, 17, 23, 2, 0, 14, 21, 24, 10, 16, 13, 6, 15, 25, 12, 22, 18, 20, 8,
                19, 3, 26, 1,
            ]),
            boards: vec![
                Board {
                    numbers: vec![
                        vec![22, 13, 17, 11, 0],
                        vec![8, 2, 23, 4, 24],
                        vec![21, 9, 14, 16, 7],
                        vec![6, 10, 3, 18, 5],
                        vec![1, 12, 20, 15, 19],
                    ],
                },
                Board {
                    numbers: vec![
                        vec![3, 15, 0, 2, 22],
                        vec![9, 18, 13, 17, 5],
                        vec![19, 8, 7, 25, 23],
                        vec![20, 11, 10, 24, 4],
                        vec![14, 21, 16, 12, 6],
                    ],
                },
                Board {
                    numbers: vec![
                        vec![14, 21, 17, 24, 4],
                        vec![10, 16, 15, 9, 19],
                        vec![18, 8, 23, 26, 20],
                        vec![22, 11, 13, 6, 5],
                        vec![2, 0, 12, 3, 7],
                    ],
                },
            ],
        };
        assert_eq!(expected, r);
    }

    #[test]
    fn sol_1_test() {
        let bingo = parse(TEST_INPUT).unwrap();
        let s = winning_solutions(&bingo).sol_1();
        assert_eq!(Some(4512), s);
    }

    #[test]
    fn sol_2_test() {
        let bingo = parse(TEST_INPUT).unwrap();
        let s = winning_solutions(&bingo).sol_2();
        assert_eq!(Some(1924), s);
    }
}

// Failures at writing combinators below.... sucks

// use combine::parser::char::*;
// use combine::stream::easy::{Error, Info};
// use combine::*;
// use std::result::Result as StdResult;
// use crate::common::usize_parser;

// finishes parsing...but produces a single board
// Bingo {
// numbers_to_draw: NumbersToDraw(vec![
//     7, 4, 9, 5, 11, 17, 23, 2, 0, 14, 21, 24, 10, 16, 13, 6, 15, 25, 12, 22, 18, 20, 8,
//     19, 3, 26, 1,
// ]),
// boards: vec![Board {
//     numbers: vec![
//         vec![22, 13, 17, 11, 0],
//         vec![8, 2, 23, 4, 24],
//         vec![21, 9, 14, 16, 7],
//         vec![6, 10, 3, 18, 5],
//         vec![1, 12, 20, 15, 19],
//         vec![3, 15, 0, 2, 22],
//         vec![9, 18, 13, 17, 5],
//         vec![19, 8, 7, 25, 23],
//         vec![20, 11, 10, 24, 4],
//         vec![14, 21, 16, 12, 6],
//         vec![14, 21, 17, 24, 4],
//         vec![10, 16, 15, 9, 19],
//         vec![18, 8, 23, 26, 20],
//         vec![22, 11, 13, 6, 5],
//         vec![2, 0, 12, 3, 7],
//     ],
// }],
// };

// Don't know why, but this parser fails because the many-board_parser doesn't eat the newline(s)
// between the boards properly...
// fn parse_old(s: &str) -> StdResult<Bingo, easy::ParseError<&str>> {
//     let numbers_to_draw_parser = sep_by(usize_parser(), char(','));
//
//     let board_parser = {
//         // let row_parser = skip_many(char(' ')).with(sep_by1(usize_parser(), skip_many1(char(' '))));
//         // let row_parser = skip_many(char(' ')).with(sep_by1(usize_parser(), skip_many1(char(' '))));
//         let row_parser = many1(skip_many(char(' ')).with(usize_parser()));
//         // gives mega big board
//         // sep_by1(spaces().with(row_parser), newline())
//         sep_by1(row_parser, newline())
//     };
//
//     let mut parser = numbers_to_draw_parser
//         .skip(spaces())
//         .and(
//             // sep_by(skip_many1(spaces()).with(board_parser), skip_many(spaces()))
//             // many1(board_parser.skip(skip_many1(newline())))
//             sep_by(board_parser, skip_many1(newline())), // many(skip_many1(newline()).with(board_parser))
//         )
//         // .and(sep_by1(board_parser, skip_many1(newline())))
//         .and_then(
//             |(numbers_to_draw, boards): (Vec<usize>, Vec<Vec<Vec<usize>>>)| {
//                 println!("numbers_to_draw {:?}", numbers_to_draw);
//                 println!("boards {:?}", boards);
//                 let numbers = NumbersToDraw(numbers_to_draw);
//                 if let Some(first_board) = boards.get(0) {
//                     let rows_count = first_board.len();
//                     if let Some(first_row) = first_board.get(0) {
//                         let cols_count = first_row.len();
//                         if boards.iter().all(|board| {
//                             board.len() == rows_count
//                                 && board.get(0).map(|row| row.len()).unwrap_or(0) == cols_count
//                         }) {
//                             let bs = boards
//                                 .into_iter()
//                                 .map(|board| Board { numbers: board })
//                                 .collect();
//                             Ok(Bingo {
//                                 numbers_to_draw: numbers,
//                                 boards: bs,
//                             })
//                         } else {
//                             Err(Error::Unexpected(Info::Static("Non-rectangular input")))
//                         }
//                     } else {
//                         Err(Error::Unexpected(Info::Static("No columns parsed")))
//                     }
//                 } else {
//                     Err(Error::Unexpected(Info::Static("No boards parsed")))
//                 }
//             },
//         );
//     let (r, _) = parser.easy_parse(s)?;
//     Ok(r)
// }
//
// // Don't know why, but this parser fails because the many-board_parser doesn't eat the newline(s)
// // between the boards properly...
// fn parse_suckage(s: &str) -> StdResult<Bingo, easy::ParseError<&str>> {
//     let numbers_to_draw_parser = sep_by(usize_parser(), char(','));
//
//     let board_parser = {
//         // let row_parser = skip_many(char(' ')).with(sep_by1(usize_parser(), skip_many1(char(' '))));
//         // let row_parser = skip_many(char(' ')).with(sep_by1(usize_parser(), skip_many1(char(' '))));
//         let row_parser = many1(skip_many(char(' ')).with(usize_parser()));
//         // gives mega big board
//         // sep_by1(spaces().with(row_parser), newline())
//         sep_by1(row_parser, newline())
//     };
//
//     let mut parser = numbers_to_draw_parser
//         .skip(spaces())
//         .and(
//             // sep_by(skip_many1(spaces()).with(board_parser), skip_many(spaces()))
//             // many1(board_parser.skip(skip_many1(newline())))
//             sep_by(board_parser, skip_many1(newline())), // many(skip_many1(newline()).with(board_parser))
//         )
//         // .and(sep_by1(board_parser, skip_many1(newline())))
//         .and_then(
//             |(numbers_to_draw, boards): (Vec<usize>, Vec<Vec<Vec<usize>>>)| {
//                 println!("numbers_to_draw {:?}", numbers_to_draw);
//                 println!("boards {:?}", boards);
//                 let numbers = NumbersToDraw(numbers_to_draw);
//                 if let Some(first_board) = boards.get(0) {
//                     let rows_count = first_board.len();
//                     if let Some(first_row) = first_board.get(0) {
//                         let cols_count = first_row.len();
//                         if boards.iter().all(|board| {
//                             board.len() == rows_count
//                                 && board.get(0).map(|row| row.len()).unwrap_or(0) == cols_count
//                         }) {
//                             let bs = boards
//                                 .into_iter()
//                                 .map(|board| Board { numbers: board })
//                                 .collect();
//                             Ok(Bingo {
//                                 numbers_to_draw: numbers,
//                                 boards: bs,
//                             })
//                         } else {
//                             Err(Error::Unexpected(Info::Static("Non-rectangular input")))
//                         }
//                     } else {
//                         Err(Error::Unexpected(Info::Static("No columns parsed")))
//                     }
//                 } else {
//                     Err(Error::Unexpected(Info::Static("No boards parsed")))
//                 }
//             },
//         );
//     let (r, _) = parser.easy_parse(s)?;
//     Ok(r)
// }
