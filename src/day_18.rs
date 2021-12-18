use std::num::ParseIntError;
use std::ops::Add;
use std::result::Result as StdResult;

use anyhow::Result;
use combine::parser::char::*;
use combine::*;
use itertools::Either;
use itertools::Either::*;

use PairTree::*;

use crate::common::usize_parser;

pub const INPUT: &str = include_str!("../data/day_18_input");

pub fn run() -> Result<()> {
    println!("*** Day 18: Snailfish ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let sol_1 = add_all_magnitude(&input);
    println!("Solution 1: {:?}", sol_1);
    let sol_2 = biggest_pair_sum(&input);
    println!("Solution 2: {:?}", sol_2);

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub struct Input(Vec<PairTree>);

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PairTree {
    Num(usize),
    Pair(Box<PairTree>, Box<PairTree>),
}

impl PairTree {
    fn pair(first: PairTree, second: PairTree) -> PairTree {
        Pair(Box::new(first), Box::new(second))
    }

    fn magnitude(&self) -> usize {
        match self {
            Num(n) => *n,
            Pair(left, right) => 3 * left.magnitude() + 2 * right.magnitude(),
        }
    }

    /// 1. Explode
    /// 2. Split
    ///
    /// If there are any changes at any step (false), invoke reduce again, otherwise advance
    fn reduce(&mut self) {
        match self.explode() {
            false => self.reduce(),
            true => match self.split() {
                false => self.reduce(),
                true => (),
            },
        }
    }

    fn explode(&mut self) -> bool {
        fn explode_subtree(depth: usize, p: &mut PairTree) -> Either<SubTreeExplosionResult, ()> {
            match (depth, &p) {
                (_, Num(_)) => Right(()), // Do nothing
                (depth, Pair(left, right)) => {
                    match (left.as_ref(), right.as_ref()) {
                        (Num(left), Num(right)) if depth >= 4 => {
                            let left_value = *left;
                            let right_value = *right;
                            *p = Num(0);
                            Left(SubTreeExplosionResult {
                                pre_explosion_left: left_value,
                                pre_explosion_right: right_value,
                            })
                        }
                        _ => {
                            // Mutation round.
                            match p {
                                Pair(left_tree, right_tree) => {
                                    // Explode left first according to instructions
                                    match explode_subtree(depth + 1, left_tree) {
                                        Left(SubTreeExplosionResult {
                                            pre_explosion_left,
                                            pre_explosion_right,
                                        }) => {
                                            add_to_leftmost(right_tree, pre_explosion_right);
                                            Left(SubTreeExplosionResult {
                                                pre_explosion_left,
                                                pre_explosion_right: 0,
                                            })
                                        }
                                        Right(()) => match explode_subtree(depth + 1, right_tree) {
                                            Left(SubTreeExplosionResult {
                                                pre_explosion_left,
                                                pre_explosion_right,
                                            }) => {
                                                add_to_rightmost(left_tree, pre_explosion_left);
                                                Left(SubTreeExplosionResult {
                                                    pre_explosion_left: 0,
                                                    pre_explosion_right,
                                                })
                                            }
                                            Right(()) => Right(()),
                                        },
                                    }
                                }
                                _ => unreachable!(), // already matched on Num
                            }
                        }
                    }
                }
            }
        }

        struct SubTreeExplosionResult {
            pre_explosion_left: usize,
            pre_explosion_right: usize,
        }

        fn add_to_leftmost(pair: &mut PairTree, num: usize) {
            if num != 0 {
                match pair {
                    Num(current) => *current += num,
                    Pair(left, _right) => add_to_leftmost(left, num),
                }
            }
        }

        fn add_to_rightmost(pair: &mut PairTree, num: usize) {
            if num != 0 {
                match pair {
                    Num(current) => *current += num,
                    Pair(_left, right) => add_to_rightmost(right, num),
                }
            }
        }

        explode_subtree(0, self).is_right()
    }

    fn split(&mut self) -> bool {
        match &self {
            Num(n) => {
                if *n >= 10 {
                    *self = PairTree::pair(
                        Num((*n as f64 / 2f64).floor() as usize),
                        Num((*n as f64 / 2f64).ceil() as usize),
                    );
                    false
                } else {
                    true
                }
            }
            _ => {
                // Mutation round
                match self {
                    Pair(left, right) => match left.split() {
                        false => false,
                        true => match right.split() {
                            false => false,
                            true => true,
                        },
                    },
                    _ => unreachable!(), // already matched on Num
                }
            }
        }
    }
}

impl Add for PairTree {
    type Output = PairTree;

    fn add(self, rhs: Self) -> Self::Output {
        let mut paired = PairTree::pair(self, rhs);
        paired.reduce();
        paired
    }
}

fn add_all_magnitude(input: &Input) -> Option<usize> {
    let final_tree = input.0.clone().into_iter().reduce(|acc, next| acc + next);
    final_tree.map(|p| p.magnitude())
}

pub fn biggest_pair_sum(input: &Input) -> Option<usize> {
    let pair_trees = &input.0;
    pair_trees
        .iter()
        .flat_map(|first_pair| {
            pair_trees.iter().filter_map(move |second_pair| {
                if first_pair != second_pair {
                    let added = first_pair.clone() + second_pair.clone();
                    Some(added.magnitude())
                } else {
                    None
                }
            })
        })
        .max()
}

pub fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
    let mut parser = many1(pair_tree().skip(spaces())).map(Input);
    let (r, _) = parser.easy_parse(s)?;
    Ok(r)
}

// As this expression parser needs to be able to call itself recursively `impl Parser` can't
// be used on its own as that would cause an infinitely large type. We can avoid this by using
// the `parser!` macro which erases the inner type and the size of that type entirely which
// lets it be used recursively.
//
// (This macro does not use `impl Trait` which means it can be used in rust < 1.26 as well to
// emulate `impl Parser`)
parser! {
    fn pair_tree[Input]()(Input) -> PairTree
    where [Input: Stream<Token = char>,
    // Necessary due to rust-lang/rust#24159
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
          <<Input as StreamOnce>::Error as combine::ParseError<
              char,
              <Input as StreamOnce>::Range,
              <Input as StreamOnce>::Position,
          >>::StreamError: From<ParseIntError>,
          <<Input as StreamOnce>::Error as combine::ParseError<
              char,
              <Input as StreamOnce>::Range,
              <Input as StreamOnce>::Position,
          >>::StreamError: From<ParseIntError>,
          <Input as combine::StreamOnce>::Error: combine::ParseError<
              char,
              <Input as combine::StreamOnce>::Range,
              <Input as combine::StreamOnce>::Position,
          >]
    {
        pair_tree_()
    }
}

// `impl Parser` can be used to create reusable parsers with zero overhead
fn pair_tree_<Input>() -> impl Parser<Input, Output = PairTree>
where
    Input: Stream<Token = char>,
    // Necessary due to rust-lang/rust#24159
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    <<Input as StreamOnce>::Error as combine::ParseError<
        char,
        <Input as StreamOnce>::Range,
        <Input as StreamOnce>::Position,
    >>::StreamError: From<ParseIntError>,
    <<Input as StreamOnce>::Error as combine::ParseError<
        char,
        <Input as StreamOnce>::Range,
        <Input as StreamOnce>::Position,
    >>::StreamError: From<ParseIntError>,
    <Input as combine::StreamOnce>::Error: combine::ParseError<
        char,
        <Input as combine::StreamOnce>::Range,
        <Input as combine::StreamOnce>::Position,
    >,
{
    let num = usize_parser();

    //Creates a parser which parses a char and skips any trailing whitespace
    let lex_char = |c| char(c);
    //We can use tuples to run several parsers in sequence
    //The resulting type is a tuple containing each parsers output
    let pair = (
        lex_char('['),
        pair_tree(),
        lex_char(','),
        pair_tree(),
        lex_char(']'),
    )
        .map(|t| PairTree::pair(t.1, t.3));

    choice((num.map(Num), pair))
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT_EXPLODE_1: &str = "[[[[[9,8],1],2],3],4]";

    static TEST_INPUT_MULTI: &str = "[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
[[[5,[2,8]],4],[5,[[9,9],0]]]
[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
[[[[5,4],[7,7]],8],[[8,3],8]]
[[9,3],[[9,9],[6,[4,9]]]]
[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]";

    #[test]
    fn parse_single_test() {
        let i = parse(TEST_INPUT_EXPLODE_1).unwrap();
        assert_eq!(
            Input(vec![PairTree::pair(
                PairTree::pair(
                    PairTree::pair(
                        PairTree::pair(PairTree::pair(Num(9), Num(8)), Num(1)),
                        Num(2),
                    ),
                    Num(3),
                ),
                Num(4),
            )]),
            i
        );
    }

    #[test]
    fn add_all_magnitude_test() {
        let i = parse(TEST_INPUT_MULTI).unwrap();
        let r = add_all_magnitude(&i);
        assert_eq!(Some(4140), r)
    }

    #[test]
    fn add_all_magnitude_real_test() {
        let i = parse(INPUT).unwrap();
        let r = add_all_magnitude(&i);
        assert_eq!(Some(3524), r)
    }

    #[test]
    fn biggest_pair_sum_test() {
        let i = parse(TEST_INPUT_MULTI).unwrap();
        let r = biggest_pair_sum(&i);
        assert_eq!(Some(3993), r)
    }

    #[test]
    fn biggest_pair_sum_real_test() {
        let i = parse(INPUT).unwrap();
        let r = biggest_pair_sum(&i);
        assert_eq!(Some(4656), r)
    }

    #[test]
    fn parse_multi_test() {
        let i = parse(TEST_INPUT_MULTI).unwrap();
        assert_eq!(
            Input(vec![
                PairTree::pair(
                    PairTree::pair(
                        PairTree::pair(Num(0), PairTree::pair(Num(5), Num(8))),
                        PairTree::pair(
                            PairTree::pair(Num(1), Num(7)),
                            PairTree::pair(Num(9), Num(6)),
                        ),
                    ),
                    PairTree::pair(
                        PairTree::pair(Num(4), PairTree::pair(Num(1), Num(2))),
                        PairTree::pair(PairTree::pair(Num(1), Num(4)), Num(2)),
                    ),
                ),
                PairTree::pair(
                    PairTree::pair(
                        PairTree::pair(Num(5), PairTree::pair(Num(2), Num(8))),
                        Num(4),
                    ),
                    PairTree::pair(
                        Num(5),
                        PairTree::pair(PairTree::pair(Num(9), Num(9)), Num(0)),
                    ),
                ),
                PairTree::pair(
                    Num(6),
                    PairTree::pair(
                        PairTree::pair(
                            PairTree::pair(Num(6), Num(2)),
                            PairTree::pair(Num(5), Num(6)),
                        ),
                        PairTree::pair(
                            PairTree::pair(Num(7), Num(6)),
                            PairTree::pair(Num(4), Num(7)),
                        ),
                    ),
                ),
                PairTree::pair(
                    PairTree::pair(
                        PairTree::pair(Num(6), PairTree::pair(Num(0), Num(7))),
                        PairTree::pair(Num(0), Num(9)),
                    ),
                    PairTree::pair(
                        Num(4),
                        PairTree::pair(Num(9), PairTree::pair(Num(9), Num(0))),
                    ),
                ),
                PairTree::pair(
                    PairTree::pair(
                        PairTree::pair(Num(7), PairTree::pair(Num(6), Num(4))),
                        PairTree::pair(Num(3), PairTree::pair(Num(1), Num(3))),
                    ),
                    PairTree::pair(
                        PairTree::pair(PairTree::pair(Num(5), Num(5)), Num(1)),
                        Num(9),
                    ),
                ),
                PairTree::pair(
                    PairTree::pair(
                        Num(6),
                        PairTree::pair(
                            PairTree::pair(Num(7), Num(3)),
                            PairTree::pair(Num(3), Num(2)),
                        ),
                    ),
                    PairTree::pair(
                        PairTree::pair(
                            PairTree::pair(Num(3), Num(8)),
                            PairTree::pair(Num(5), Num(7)),
                        ),
                        Num(4),
                    ),
                ),
                PairTree::pair(
                    PairTree::pair(
                        PairTree::pair(
                            PairTree::pair(Num(5), Num(4)),
                            PairTree::pair(Num(7), Num(7)),
                        ),
                        Num(8),
                    ),
                    PairTree::pair(PairTree::pair(Num(8), Num(3)), Num(8)),
                ),
                PairTree::pair(
                    PairTree::pair(Num(9), Num(3)),
                    PairTree::pair(
                        PairTree::pair(Num(9), Num(9)),
                        PairTree::pair(Num(6), PairTree::pair(Num(4), Num(9))),
                    ),
                ),
                PairTree::pair(
                    PairTree::pair(
                        Num(2),
                        PairTree::pair(PairTree::pair(Num(7), Num(7)), Num(7)),
                    ),
                    PairTree::pair(
                        PairTree::pair(Num(5), Num(8)),
                        PairTree::pair(
                            PairTree::pair(Num(9), Num(3)),
                            PairTree::pair(Num(0), Num(2)),
                        ),
                    ),
                ),
                PairTree::pair(
                    PairTree::pair(
                        PairTree::pair(PairTree::pair(Num(5), Num(2)), Num(5)),
                        PairTree::pair(Num(8), PairTree::pair(Num(3), Num(7))),
                    ),
                    PairTree::pair(
                        PairTree::pair(Num(5), PairTree::pair(Num(7), Num(5))),
                        PairTree::pair(Num(4), Num(4)),
                    ),
                ),
            ]),
            i
        );
    }
}
