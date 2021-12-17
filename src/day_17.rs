use std::result::Result as StdResult;

use anyhow::Result;
use combine::parser::char::*;
use combine::*;

use crate::common::isize_parser;

pub const INPUT: &str = include_str!("../data/day_17_input");

pub fn run() -> Result<()> {
    println!("*** Day 17: Trick Shot ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let sol_1 = input.highest_height_that_hits_target();
    println!("Solution 1: {:?}", sol_1);
    let sol_2 = input.distinct_velocities_that_hit_target();
    println!("Solution 2: {:?}", sol_2);

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub struct Input {
    target_x_min: X,
    target_x_max: X,
    target_y_min: Y,
    target_y_max: Y,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct X(isize);
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Y(isize);
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct VX(isize);
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct VY(isize);

impl Input {
    pub fn highest_height_that_hits_target(&self) -> Option<Y> {
        let mut valid_velocities = self
            .init_velocities_that_land_in_box()
            .map(|(_, v_y)| v_y)
            .collect::<Vec<_>>();
        valid_velocities.sort_by(|v_y_1, v_1_2| v_y_1.cmp(v_1_2).reverse());
        let max_v_y_init = valid_velocities.first()?;

        Input::y_positions_over_time(*max_v_y_init)
            .take_while(|y| y >= &self.target_y_min)
            .max()
    }

    pub fn distinct_velocities_that_hit_target(&self) -> usize {
        self.init_velocities_that_land_in_box().count()
    }

    fn init_velocities_that_land_in_box(&self) -> impl Iterator<Item = (VX, VY)> + '_ {
        // Speed limits are reasoned about by using the target borders and preventing instantaenous
        // overshooting...
        let max_x_speed = self.target_x_min.0.abs().max(self.target_x_max.0.abs());
        // velocity in x  eventually reaches 0, so we can assume that if it's on other side of the x
        // axis, 0 is the lowest it can be, and it must go in the direction of the target
        let min_v_x = if self.target_x_min.0 < 0 {
            -max_x_speed
        } else {
            0
        };
        let max_v_x = if self.target_x_max.0 < 0 {
            0
        } else {
            max_x_speed
        };
        let min_v_y = if self.target_y_max.0 < 0 {
            self.target_y_min.0 // otherwise, it simultaneously overshoots the target
        } else {
            0 // otherwise it will never reach the target
        };
        // velocity in y has no lower limits ; so multiply absolute thresholds by 2 Just In Case ™
        let max_v_y = self.target_y_min.0.abs().max(self.target_y_max.0.abs()) * 2;

        (min_v_x..=max_v_x).flat_map(move |v_x_init_raw| {
            (min_v_y..=max_v_y).filter_map(move |v_y_init_raw| {
                let v_x_init = VX(v_x_init_raw);
                let v_y_init = VY(v_y_init_raw);
                let results_in_trajectory_in_target = Input::x_positions_over_time(v_x_init)
                    .zip(Input::y_positions_over_time(v_y_init))
                    .take_while(|(x_pos, y_pos)| {
                        ((self.target_x_min.0 >= 0 && x_pos <= &self.target_x_max)
                            || (self.target_x_max.0 < 0 && x_pos >= &self.target_x_min))
                            && ((self.target_y_max.0 < 0 && y_pos >= &self.target_y_min)
                                || (self.target_y_min.0 >= 0 && y_pos.0 >= 0))
                    })
                    .any(|x_and_y| self.in_target(x_and_y));
                if results_in_trajectory_in_target {
                    Some((v_x_init, v_y_init))
                } else {
                    None
                }
            })
        })
    }

    fn in_target(&self, (x, y): (X, Y)) -> bool {
        self.target_x_min <= x
            && self.target_x_max >= x
            && self.target_y_min <= y
            && self.target_y_max >= y
    }

    fn x_positions_over_time(VX(init_velocity): VX) -> impl Iterator<Item = X> {
        (0..).scan((0isize, init_velocity), |(last, v), _step| {
            let next = *last + *v;
            *last = next;
            *v = match *v {
                v if v > 0 => v - 1,
                v if v < 0 => v + 1,
                _ => *v,
            };
            Some(X(next))
        })
    }

    fn y_positions_over_time(VY(init_velocity): VY) -> impl Iterator<Item = Y> {
        (0..).scan((0isize, init_velocity), |(last, v), _step| {
            let next = *last + *v;
            *last = next;
            *v -= 1;

            Some(Y(next))
        })
    }
}

pub fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
    let mut parser = string("target area: ")
        .with(
            string("x=")
                .with(isize_parser())
                .skip(string(".."))
                .and(isize_parser()),
        )
        .skip(string(", "))
        .and(
            string("y=")
                .with(isize_parser())
                .skip(string(".."))
                .and(isize_parser()),
        )
        .map(|((x1, x2), (y1, y2))| Input {
            target_x_min: X(x1.min(x2)),
            target_x_max: X(x1.max(x2)),
            target_y_min: Y(y1.min(y2)),
            target_y_max: Y(y1.max(y2)),
        });
    let (r, _) = parser.easy_parse(s)?;
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &str = "target area: x=20..30, y=-10..-5";

    #[test]
    fn parse_test() {
        let i = parse(TEST_INPUT).unwrap();
        assert_eq!(
            Input {
                target_x_min: X(20),
                target_x_max: X(30),
                target_y_min: Y(-10),
                target_y_max: Y(-5)
            },
            i
        );
    }

    #[test]
    fn highest_y_velocity_that_hits_target_test() {
        let i = parse(TEST_INPUT).unwrap();
        let r = i.highest_height_that_hits_target();
        assert_eq!(Some(Y(45)), r);
    }

    #[test]
    fn highest_y_velocity_that_hits_real_input_target_test() {
        let i = parse(INPUT).unwrap();
        let r = i.highest_height_that_hits_target();
        assert_eq!(Some(Y(5671)), r);
    }

    #[test]
    fn distinct_velocities_that_hit_target_test() {
        let i = parse(TEST_INPUT).unwrap();
        let r = i.distinct_velocities_that_hit_target();
        assert_eq!(112, r);
    }

    #[test]
    fn distinct_velocities_that_hit_target_real_input_test() {
        let i = parse(INPUT).unwrap();
        let r = i.distinct_velocities_that_hit_target();
        assert_eq!(4556, r);
    }
}
