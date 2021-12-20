use std::fmt::{Display, Formatter};
use std::result::Result as StdResult;

use anyhow::Result;
use combine::easy::{Error, Info};
use combine::parser::char::*;
use combine::*;

pub const INPUT: &str = include_str!("../data/day_20_input");

pub fn run() -> Result<()> {
    println!("*** Day 20: Trench Map ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let enhanced_twice = input.enhance(2);
    let sol_1 = enhanced_twice.image.on_pixels();
    println!("Solution 1: {:?}", sol_1);

    let enhanced_fifty = input.enhance(50);
    let sol_2 = enhanced_fifty.image.on_pixels();
    println!("Solution 2: {:?}", sol_2);

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub struct Input {
    algorithm: Vec<bool>,
    image: Image,
}

impl Input {
    pub fn enhance(&self, steps_to_take: usize) -> EnhancementState {
        let init_state = EnhancementState {
            step: 0,
            image: self.image.clone(),
        };
        (0..=steps_to_take).fold(init_state, |acc, step| {
            if step > 0 {
                let background_state = if !acc.image.background_state {
                    self.algorithm[0]
                } else {
                    self.algorithm[511 /* 111111111 in dec */]
                };
                // Just make sure to get the corners
                let rows = acc.image.rows + 4;
                let columns = acc.image.columns + 4;
                let mut bits = vec![vec![false; columns]; rows];
                for (new_image_row_idx, new_image_row) in bits.iter_mut().enumerate() {
                    for (new_image_col_idx, new_image_pixel) in new_image_row.iter_mut().enumerate()
                    {
                        let prev_image_row_idx = new_image_row_idx as isize - 2;
                        let prev_image_col_idx = new_image_col_idx as isize - 2;
                        let previous_image_coords = Coords {
                            row: prev_image_row_idx,
                            col: prev_image_col_idx,
                        };
                        let algo_idx = acc.image.get_algo_index_for_pixel_at(previous_image_coords);
                        let algo_value = self.algorithm[algo_idx];
                        *new_image_pixel = algo_value;
                    }
                }
                let image = Image {
                    bits,
                    background_state,
                    rows,
                    columns,
                };
                EnhancementState { step, image }
            } else {
                acc
            }
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Image {
    bits: Vec<Vec<bool>>,
    background_state: bool,
    rows: usize,
    columns: usize,
}

struct Coords {
    row: isize,
    col: isize,
}

impl Image {
    fn get_algo_index_for_pixel_at(&self, Coords { row, col }: Coords) -> usize {
        let idx_bits = [
            (row - 1, col - 1),
            (row - 1, col),
            (row - 1, col + 1),
            (row, col - 1),
            (row, col),
            (row, col + 1),
            (row + 1, col - 1),
            (row + 1, col),
            (row + 1, col + 1),
        ]
        .map(|(r, c)| {
            if r >= 0isize && r < self.rows as isize && c >= 0isize && c < self.columns as isize {
                self.bits[r as usize][c as usize]
            } else {
                self.background_state
            }
        });

        to_decimal(&idx_bits)
    }

    pub fn on_pixels(&self) -> usize {
        self.bits
            .iter()
            .map(|row| row.iter().filter(|v| **v).count())
            .sum()
    }
}

impl Display for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (row_idx, row) in self.bits.iter().enumerate() {
            for pixel in row {
                if *pixel {
                    write!(f, "#")?
                } else {
                    write!(f, ".")?
                }
            }
            if row_idx < self.rows - 1 {
                writeln!(f)?
            }
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct EnhancementState {
    step: usize,
    image: Image,
}

impl EnhancementState {
    pub fn image(&self) -> &Image {
        &self.image
    }
}

fn to_decimal(bits: &[bool]) -> usize {
    let mut acc = 0;
    for (idx, next) in bits.iter().rev().enumerate() {
        if *next {
            acc += 2usize.pow(idx as u32);
        }
    }
    acc
}

pub fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
    let dots_parser = || many1(char('#').map(|_| true).or(char('.').map(|_| false)));

    let image_parser = many1(dots_parser().skip(spaces())).and_then(|vecs: Vec<Vec<bool>>| {
        if let Some(first) = vecs.get(0) {
            let rows = vecs.len();
            let columns = first.len();
            if vecs.iter().all(|v| v.len() == columns) {
                Ok(Image {
                    bits: vecs,
                    background_state: false,
                    rows,
                    columns,
                })
            } else {
                Err(Error::Unexpected(Info::Static("Non-rectangular input")))
            }
        } else {
            Err(Error::Unexpected(Info::Static("No input")))
        }
    });

    let mut parser = dots_parser()
        .skip(spaces())
        .and(image_parser)
        .map(|(algorithm, image)| Input { algorithm, image });
    let (r, _) = parser.easy_parse(s)?;
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &str = "..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##..###..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#......#.###.######.###.####...#.##.##..#..#..#####.....#.#....###..#.##......#.....#..#..#..##..#...##.######.####.####.#.#...#.......#..#.#.#...####.##.#......#..#...##.#.##..#...##.#.##..###.#......#.#.......#.#.#.####.###.##...#.....####.#..#..#.##.#....##..#.####....##...##..#...#......#.#.......#.......##..####..#...#.#.#...##..#.#..###..#####........#..####......#..#

#..#.
#....
##..#
..#..
..###";

    #[test]
    fn input_parse_test() {
        let r = parse(TEST_INPUT).unwrap();
        let expected = Input {
            algorithm: vec![
                false, false, true, false, true, false, false, true, true, true, true, true, false,
                true, false, true, false, true, false, true, true, true, false, true, true, false,
                false, false, false, false, true, true, true, false, true, true, false, true,
                false, false, true, true, true, false, true, true, true, true, false, false, true,
                true, true, true, true, false, false, true, false, false, false, false, true,
                false, false, true, false, false, true, true, false, false, true, true, true,
                false, false, true, true, true, true, true, true, false, true, true, true, false,
                false, false, true, true, true, true, false, false, true, false, false, true, true,
                true, true, true, false, false, true, true, false, false, true, false, true, true,
                true, true, true, false, false, false, true, true, false, true, false, true, false,
                false, true, false, true, true, false, false, true, false, true, false, false,
                false, false, false, false, true, false, true, true, true, false, true, true, true,
                true, true, true, false, true, true, true, false, true, true, true, true, false,
                false, false, true, false, true, true, false, true, true, false, false, true,
                false, false, true, false, false, true, true, true, true, true, false, false,
                false, false, false, true, false, true, false, false, false, false, true, true,
                true, false, false, true, false, true, true, false, false, false, false, false,
                false, true, false, false, false, false, false, true, false, false, true, false,
                false, true, false, false, true, true, false, false, true, false, false, false,
                true, true, false, true, true, true, true, true, true, false, true, true, true,
                true, false, true, true, true, true, false, true, false, true, false, false, false,
                true, false, false, false, false, false, false, false, true, false, false, true,
                false, true, false, true, false, false, false, true, true, true, true, false, true,
                true, false, true, false, false, false, false, false, false, true, false, false,
                true, false, false, false, true, true, false, true, false, true, true, false,
                false, true, false, false, false, true, true, false, true, false, true, true,
                false, false, true, true, true, false, true, false, false, false, false, false,
                false, true, false, true, false, false, false, false, false, false, false, true,
                false, true, false, true, false, true, true, true, true, false, true, true, true,
                false, true, true, false, false, false, true, false, false, false, false, false,
                true, true, true, true, false, true, false, false, true, false, false, true, false,
                true, true, false, true, false, false, false, false, true, true, false, false,
                true, false, true, true, true, true, false, false, false, false, true, true, false,
                false, false, true, true, false, false, true, false, false, false, true, false,
                false, false, false, false, false, true, false, true, false, false, false, false,
                false, false, false, true, false, false, false, false, false, false, false, true,
                true, false, false, true, true, true, true, false, false, true, false, false,
                false, true, false, true, false, true, false, false, false, true, true, false,
                false, true, false, true, false, false, true, true, true, false, false, true, true,
                true, true, true, false, false, false, false, false, false, false, false, true,
                false, false, true, true, true, true, false, false, false, false, false, false,
                true, false, false, true,
            ],
            image: Image {
                bits: vec![
                    vec![true, false, false, true, false],
                    vec![true, false, false, false, false],
                    vec![true, true, false, false, true],
                    vec![false, false, true, false, false],
                    vec![false, false, true, true, true],
                ],
                rows: 5,
                background_state: false,
                columns: 5,
            },
        };
        assert_eq!(expected, r);
    }

    #[test]
    fn to_decimal_test() {
        let r = to_decimal(&[false]);
        assert_eq!(0, r);
        let r = to_decimal(&[true]);
        assert_eq!(1, r);
        let r = to_decimal(&[true, false]);
        assert_eq!(2, r);
        let r = to_decimal(&[true, true, true]);
        assert_eq!(7, r);
    }

    #[test]
    fn enhance_test() {
        let i = parse(TEST_INPUT).unwrap();
        let enhanced_twice = i.enhance(2);
        assert_eq!(35, enhanced_twice.image.on_pixels());
        let enhanced_fifty = i.enhance(50);
        assert_eq!(3351, enhanced_fifty.image.on_pixels());
    }

    #[test]
    fn get_algo_index_for_pixel_at_test() {
        let r = parse(TEST_INPUT).unwrap();
        let image = r.image;
        // nether regions

        let expected_background_num = to_decimal(&vec![image.background_state; 9]);
        let r = image.get_algo_index_for_pixel_at(Coords {
            row: isize::MAX / 2,
            col: isize::MAX / 2,
        });
        assert_eq!(expected_background_num, r);
        let r = image.get_algo_index_for_pixel_at(Coords {
            row: isize::MAX / 2,
            col: isize::MIN / 2,
        });
        assert_eq!(expected_background_num, r);
        let r = image.get_algo_index_for_pixel_at(Coords {
            row: isize::MIN / 2,
            col: isize::MAX / 2,
        });
        assert_eq!(expected_background_num, r);
        let r = image.get_algo_index_for_pixel_at(Coords {
            row: isize::MIN / 2,
            col: isize::MIN / 2,
        });
        assert_eq!(expected_background_num, r);

        let r = image.get_algo_index_for_pixel_at(Coords { row: 0, col: 0 });
        assert_eq!(
            to_decimal(&[false, false, false, false, true, false, false, true, false,]),
            r
        );
        // upper left corner only
        let r = image.get_algo_index_for_pixel_at(Coords { row: -1, col: -1 });
        assert_eq!(
            to_decimal(&[false, false, false, false, false, false, false, false, true,]),
            r
        ); // just the dot in the upper left

        // middle
        let r = image.get_algo_index_for_pixel_at(Coords { row: 2, col: 2 });
        assert_eq!(34, r)
    }
}
