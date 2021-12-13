use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use anyhow::{Context, Result};
use combine::parser::char::*;
use combine::*;

use crate::common::usize_parser;

pub const INPUT: &str = include_str!("../data/day_13_input");

pub fn run() -> Result<()> {
    println!("*** Day 13: Transparent Origami ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let first_fold_along = input
        .fold_alongs
        .first()
        .context("Should have a first fold !??!")?;
    let sol_1 = fold_along(&input.dots, &input.max_coords, first_fold_along);
    println!("Solution 1: {:?}", sol_1.dots.len());

    let sol_2 = input.fold_all();
    println!("Solution 2");
    println!("{}", sol_2);
    Ok(())
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
pub struct Coords {
    x: usize,
    y: usize,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum FoldAlong {
    X(usize),
    Y(usize),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Input {
    dots: HashSet<Coords>,
    max_coords: Coords,
    fold_alongs: Vec<FoldAlong>,
}

impl Input {
    pub fn fold_all(&self) -> FoldOutput {
        let (dots, max_coords) = self.fold_alongs.iter().fold(
            (self.dots.clone(), self.max_coords),
            |(dots_acc, max_coods_acc), next_fold| {
                let r = fold_along(&dots_acc, &max_coods_acc, next_fold);
                (r.dots, r.max_coords)
            },
        );
        FoldOutput { dots, max_coords }
    }
}

pub struct FoldOutput {
    pub dots: HashSet<Coords>,
    max_coords: Coords,
}

impl Display for FoldOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..=self.max_coords.y {
            for x in 0..=self.max_coords.x {
                if self.dots.contains(&Coords { x, y }) {
                    write!(f, "#")?;
                } else {
                    write!(f, ".")?;
                }
            }
            if y != self.max_coords.y {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

fn fold_along(dots: &HashSet<Coords>, max_coords: &Coords, fold_along: &FoldAlong) -> FoldOutput {
    match *fold_along {
        FoldAlong::Y(fold_on_y) => {
            let min_y_of_section_to_fold_with = fold_on_y + 1;
            let max_y_of_section_folded_onto = fold_on_y - 1;
            let height_of_fold_up_section = max_coords.y - min_y_of_section_to_fold_with + 1;

            let mut dots_in_fold_up = (min_y_of_section_to_fold_with..=max_coords.y).fold(
                HashSet::new(),
                |mut acc, next_y| {
                    let dots_with_y = dots.iter().filter(|Coords { y, .. }| *y == next_y);
                    let new_y = 2 * fold_on_y - next_y;
                    let dots_at_line_with_new_y =
                        dots_with_y.map(|Coords { x, .. }| Coords { x: *x, y: new_y });
                    acc.extend(dots_at_line_with_new_y);
                    acc
                },
            );

            let dots_folded_onto = dots.iter().filter(|Coords { y, .. }| *y < fold_on_y);
            dots_in_fold_up.extend(dots_folded_onto);

            let new_max_y_after_fold =
                max_y_of_section_folded_onto.max(height_of_fold_up_section - 1);
            let new_max_coords = Coords {
                x: max_coords.x,
                y: new_max_y_after_fold,
            };

            FoldOutput {
                dots: dots_in_fold_up,
                max_coords: new_max_coords,
            }
        }
        FoldAlong::X(fold_on_x) => {
            let min_x_of_section_to_fold_with = fold_on_x + 1;
            let width_of_section_folded_with = max_coords.x - min_x_of_section_to_fold_with + 1;
            let max_x_of_section_to_fold_onto = fold_on_x - 1;

            let mut dots_in_fold_right = (min_x_of_section_to_fold_with..=max_coords.x).fold(
                HashSet::new(),
                |mut acc, next_x| {
                    let dots_with_x = dots.iter().filter(|Coords { x, .. }| *x == next_x);
                    let new_x = fold_on_x * 2 - next_x;
                    let dots_at_line_with_new_x =
                        dots_with_x.map(|Coords { y, .. }| Coords { x: new_x, y: *y });
                    acc.extend(dots_at_line_with_new_x);
                    acc
                },
            );
            let dots_folded_onto = dots.iter().filter(|Coords { x, .. }| *x < fold_on_x);
            dots_in_fold_right.extend(dots_folded_onto);

            let new_max_x_after_fold =
                max_x_of_section_to_fold_onto.max(width_of_section_folded_with - 1);
            let new_max_coords = Coords {
                x: new_max_x_after_fold,
                y: max_coords.y,
            };

            FoldOutput {
                dots: dots_in_fold_right,
                max_coords: new_max_coords,
            }
        }
    }
}

fn max_coords(dots: &HashSet<Coords>) -> Coords {
    let (x, y) = dots.iter().fold((0, 0), |(acc_x, acc_y), Coords { x, y }| {
        (acc_x.max(*x), acc_y.max(*y))
    });
    Coords { x, y }
}

// Ugh, the double newline separator still messes with me...
// TODO: try with pure Combine, again (https://gitter.im/Marwes/combine?at=5fde845622f12e449bfe9459)
pub fn parse(s: &str) -> Result<Input> {
    let mut split_by_newline = s.trim().split("\n\n"); // ugh.... really need to figure out how to do this with just combine...

    let dots_str = split_by_newline.next().context("Dots section found")?;

    let dots: HashSet<Coords> = dots_str
        .lines()
        .filter_map(|dot_str| {
            let mut dot_parser = usize_parser()
                .skip(char(','))
                .and(usize_parser())
                .map(|(x, y)| Coords { x, y });
            let (dot, _) = dot_parser.easy_parse(dot_str.trim()).ok()?;
            Some(dot)
        })
        .collect();

    let max_coords = max_coords(&dots);

    let fold_alongs_str = split_by_newline
        .next()
        .context("No fold along section found")?;

    let fold_alongs = fold_alongs_str
        .lines()
        .filter_map(|fold_along_str| {
            let mut fold_along_parser = string("fold along ").with(
                attempt(string("x=").with(usize_parser()))
                    .map(FoldAlong::X)
                    .or(attempt(
                        attempt(string("y=").with(usize_parser())).map(FoldAlong::Y),
                    )),
            );
            let (fold_along, _) = fold_along_parser.easy_parse(fold_along_str.trim()).ok()?;
            Some(fold_along)
        })
        .collect();

    let r = Input {
        dots,
        max_coords,
        fold_alongs,
    };
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;
    use FoldAlong::*;

    static TEST_INPUT: &str = "6,10
0,14
9,10
0,3
10,4
4,11
6,0
6,12
4,1
0,13
10,12
3,4
3,0
8,4
1,10
2,14
8,10
9,0

fold along y=7
fold along x=5";

    #[test]
    fn parse_test() {
        let i = parse(TEST_INPUT).unwrap();
        assert_eq!(
            Input {
                dots: HashSet::from_iter([
                    Coords { x: 6, y: 10 },
                    Coords { x: 0, y: 14 },
                    Coords { x: 9, y: 10 },
                    Coords { x: 0, y: 3 },
                    Coords { x: 10, y: 4 },
                    Coords { x: 4, y: 11 },
                    Coords { x: 6, y: 0 },
                    Coords { x: 6, y: 12 },
                    Coords { x: 4, y: 1 },
                    Coords { x: 0, y: 13 },
                    Coords { x: 10, y: 12 },
                    Coords { x: 3, y: 4 },
                    Coords { x: 3, y: 0 },
                    Coords { x: 8, y: 4 },
                    Coords { x: 1, y: 10 },
                    Coords { x: 2, y: 14 },
                    Coords { x: 8, y: 10 },
                    Coords { x: 9, y: 0 }
                ]),
                max_coords: Coords { x: 10, y: 14 },
                fold_alongs: vec![Y(7), X(5)],
            },
            i
        )
    }

    #[test]
    fn fold_along_test() {
        let i = parse(TEST_INPUT).unwrap();
        let r = fold_along(&i.dots, &i.max_coords, &Y(7));

        assert_eq!(
            HashSet::from_iter([
                Coords { x: 0, y: 0 },
                Coords { x: 0, y: 1 },
                Coords { x: 0, y: 3 },
                Coords { x: 1, y: 4 },
                Coords { x: 2, y: 0 },
                Coords { x: 3, y: 0 },
                Coords { x: 3, y: 4 },
                Coords { x: 4, y: 1 },
                Coords { x: 4, y: 3 },
                Coords { x: 6, y: 0 },
                Coords { x: 6, y: 2 },
                Coords { x: 6, y: 4 },
                Coords { x: 8, y: 4 },
                Coords { x: 9, y: 0 },
                Coords { x: 9, y: 4 },
                Coords { x: 10, y: 2 },
                Coords { x: 10, y: 4 }
            ]),
            r.dots
        );
        println!("{}", r);
        assert_eq!(17, r.dots.len());

        let r = fold_along(&r.dots, &r.max_coords, &X(5));
        println!("{}", r);
        assert_eq!(16, r.dots.len());

        let r = fold_along(&r.dots, &r.max_coords, &Y(3));
        println!("{}", r);
        assert_eq!(12, r.dots.len());

        let r = fold_along(&r.dots, &r.max_coords, &X(2));
        println!("{}", r);
        assert_eq!(5, r.dots.len());
    }
}
