use std::fmt::{Display, Formatter};
use std::result::Result as StdResult;

use anyhow::Result;
use combine::easy;
use combine::parser::char::*;
use combine::*;

use crate::common::usize_parser;

const INPUT: &str = include_str!("../data/day_05_input");

pub fn run() -> Result<()> {
    println!("*** Day 5: Hydrothermal Venture ***");
    println!("Input: {}", INPUT);
    let input = Input::parse(INPUT)?;
    let part_1_diagram = part_1_diagram(&input);
    println!("Solution 1: {:?}\n", part_1_diagram.count_overlaps());
    let part_2_diagram = part_2_diagram(&input);
    println!("Solution 2: {:?}\n", part_2_diagram.count_overlaps());
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Point {
    x: usize,
    y: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct Line {
    start: Point,
    end: Point,
}

#[derive(Debug, Eq, PartialEq)]
struct Input {
    lines: Vec<Line>,
    max: Point,
}

#[derive(Eq, PartialEq)]
struct Diagram(Vec<Vec<usize>>);

impl Diagram {
    fn count_overlaps(&self) -> usize {
        self.0
            .iter()
            .flat_map(|row| row.iter().filter(|dot| **dot >= 2))
            .count()
    }
}

impl Display for Diagram {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let max_y = self.0.len();
        let max_x = self.0.first().map(|r| r.len()).unwrap_or(0);
        for y in 0..max_y {
            for x in 0..max_x {
                if let Some(dot) = self.0.get(x).and_then(|r| r.get(y)) {
                    match dot {
                        0 => write!(f, ".")?,
                        other => write!(f, "{}", other)?,
                    }
                }
            }
            if y != max_y - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl Input {
    fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
        macro_rules! point_parser {
            () => {
                usize_parser().skip(char(',')).and(usize_parser())
            };
        }
        let line_parser = point_parser!()
            .skip(space())
            .skip(string("->"))
            .skip(space())
            .and(point_parser!())
            .map(|((start_x, start_y), (end_x, end_y))| Line {
                start: Point {
                    x: start_x,
                    y: start_y,
                },
                end: Point { x: end_x, y: end_y },
            });
        let mut parser = many1(line_parser.skip(spaces())).map(|lines: Vec<Line>| {
            let max = max_point(lines.as_slice());
            Input { lines, max }
        });
        let (r, _) = parser.easy_parse(s)?;
        Ok(r)
    }
}

fn max_point(lines: &[Line]) -> Point {
    lines.iter().fold(Point { x: 0, y: 0 }, |acc, next| {
        let max_x = acc.x.max(next.start.x).max(next.end.x);
        let max_y = acc.x.max(next.start.y).max(next.end.y);
        Point { x: max_x, y: max_y }
    })
}

fn part_1_diagram(i: &Input) -> Diagram {
    build_diagram(i, |_| ())
}

fn part_2_diagram(i: &Input) -> Diagram {
    build_diagram(i, |(acc, start, end)| {
        let is_diagonal = {
            let x_diff = start.x.max(end.x) - start.x.min(end.x);
            let y_diff = start.y.max(end.y) - start.y.min(end.y);
            x_diff == y_diff
        };
        if is_diagonal {
            // Lock-step iteration where we possibly need it to go backwards is a PITA
            // in rust because reversed iterators are a different type and you can't
            // step_by negative 😭
            let x_incr: isize = if start.x < end.x { 1 } else { -1 };
            let y_incr: isize = if start.y < end.y { 1 } else { -1 };

            let mut x = start.x;
            let mut y = start.y;
            loop {
                acc[x][y] += 1;
                x = (x as isize + x_incr) as usize;
                y = (y as isize + y_incr) as usize;
                if (x, y) == (end.x, end.y) {
                    acc[x][y] += 1;
                    break;
                }
            }
        }
    })
}

#[allow(clippy::needless_range_loop)]
fn build_diagram<F>(i: &Input, handle_non_vert_or_horiz_line: F) -> Diagram
where
    F: Fn((&mut Vec<Vec<usize>>, &Point, &Point)),
{
    let d = i.lines.iter().fold(
        vec![vec![0; i.max.y + 1]; i.max.x + 1],
        |mut acc, Line { start, end }| {
            if start.x == end.x {
                // mark column
                for y in start.y.min(end.y)..=start.y.max(end.y) {
                    acc[start.x][y] += 1
                }
            } else if start.y == end.y {
                // mark row
                for x in start.x.min(end.x)..=start.x.max(end.x) {
                    acc[x][start.y] += 1
                }
            } else {
                handle_non_vert_or_horiz_line((&mut acc, start, end));
            }
            acc
        },
    );
    Diagram(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &str = "0,9 -> 5,9
8,0 -> 0,8
9,4 -> 3,4
2,2 -> 2,1
7,0 -> 7,4
6,4 -> 2,0
0,9 -> 2,9
3,4 -> 1,4
0,0 -> 8,8
5,5 -> 8,2";

    #[test]
    fn input_parse_test() {
        let r = Input::parse(TEST_INPUT).unwrap();
        let expected = Input {
            lines: vec![
                Line {
                    start: Point { x: 0, y: 9 },
                    end: Point { x: 5, y: 9 },
                },
                Line {
                    start: Point { x: 8, y: 0 },
                    end: Point { x: 0, y: 8 },
                },
                Line {
                    start: Point { x: 9, y: 4 },
                    end: Point { x: 3, y: 4 },
                },
                Line {
                    start: Point { x: 2, y: 2 },
                    end: Point { x: 2, y: 1 },
                },
                Line {
                    start: Point { x: 7, y: 0 },
                    end: Point { x: 7, y: 4 },
                },
                Line {
                    start: Point { x: 6, y: 4 },
                    end: Point { x: 2, y: 0 },
                },
                Line {
                    start: Point { x: 0, y: 9 },
                    end: Point { x: 2, y: 9 },
                },
                Line {
                    start: Point { x: 3, y: 4 },
                    end: Point { x: 1, y: 4 },
                },
                Line {
                    start: Point { x: 0, y: 0 },
                    end: Point { x: 8, y: 8 },
                },
                Line {
                    start: Point { x: 5, y: 5 },
                    end: Point { x: 8, y: 2 },
                },
            ],
            max: Point { x: 9, y: 9 },
        };
        assert_eq!(expected, r);
    }

    #[test]
    fn part_1_diagram_test() {
        let r = Input::parse(TEST_INPUT).unwrap();
        let diagram = part_1_diagram(&r);
        let expected = ".......1..
..1....1..
..1....1..
.......1..
.112111211
..........
..........
..........
..........
222111....";
        let diagram_s = format!("{}", diagram);
        assert_eq!(expected, diagram_s);
        let part_1_sol = diagram.count_overlaps();
        assert_eq!(5, part_1_sol);
    }

    #[test]
    fn part_2_diagram_test() {
        let r = Input::parse(TEST_INPUT).unwrap();
        let diagram = part_2_diagram(&r);
        let expected = "1.1....11.
.111...2..
..2.1.111.
...1.2.2..
.112313211
...1.2....
..1...1...
.1.....1..
1.......1.
222111....";
        let diagram_s = format!("{}", diagram);
        assert_eq!(expected, diagram_s);
        assert_eq!(12, diagram.count_overlaps());
    }
}
