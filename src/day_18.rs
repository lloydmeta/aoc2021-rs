use std::num::ParseIntError;
use std::result::Result as StdResult;

use anyhow::Result;
use combine::parser::char::*;
use combine::*;

use crate::common::usize_parser;
use itertools::Either;
use itertools::Either::*;
use std::ops::Add;
use PairTree::*;

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

    fn reduce(self) -> PairTree {
        match self.explode().right_and_then(|p| p.split()) {
            Left(new) => new.reduce(),
            Right(no_change) => no_change,
        }
    }

    fn explode(self) -> Either<PairTree, PairTree> {
        fn explode_subtree(depth: usize, p: PairTree) -> Either<SubTreeExplosionResult, PairTree> {
            match (depth, p) {
                (_, Num(v)) => Right(Num(v)), // Do nothing
                (depth, Pair(left, right)) => {
                    match (*left, *right) {
                        (Num(left), Num(right)) if depth >= 4 => {
                            // Explode, return 0 as the new Tree
                            Left(SubTreeExplosionResult {
                                pre_explosion_left: left,
                                new_subtree: Num(0),
                                pre_explosion_right: right,
                            })
                        }
                        (left_tree, right_tree) => {
                            // Explode left first according to instructions
                            match explode_subtree(depth + 1, left_tree) {
                                Left(SubTreeExplosionResult {
                                    pre_explosion_left,
                                    new_subtree,
                                    pre_explosion_right,
                                }) => {
                                    let new_subtree = PairTree::pair(
                                        new_subtree,
                                        add_to_leftmost(right_tree, pre_explosion_right),
                                    );
                                    Left(SubTreeExplosionResult {
                                        pre_explosion_left,
                                        new_subtree,
                                        pre_explosion_right: 0,
                                    })
                                }
                                Right(unchanged_left) => {
                                    match explode_subtree(depth + 1, right_tree) {
                                        Left(SubTreeExplosionResult {
                                            pre_explosion_left,
                                            new_subtree,
                                            pre_explosion_right,
                                        }) => {
                                            let new_subtree = PairTree::pair(
                                                add_to_rightmost(
                                                    unchanged_left,
                                                    pre_explosion_left,
                                                ),
                                                new_subtree,
                                            );
                                            Left(SubTreeExplosionResult {
                                                pre_explosion_left: 0,
                                                new_subtree,
                                                pre_explosion_right,
                                            })
                                        }
                                        Right(unchanged_right) => {
                                            Right(PairTree::pair(unchanged_left, unchanged_right))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        struct SubTreeExplosionResult {
            pre_explosion_left: usize,
            new_subtree: PairTree,
            pre_explosion_right: usize,
        }

        fn add_to_leftmost(pair: PairTree, num: usize) -> PairTree {
            if num != 0 {
                match pair {
                    Num(current) => Num(current + num),
                    Pair(left, right) => PairTree::pair(add_to_leftmost(*left, num), *right),
                }
            } else {
                pair
            }
        }

        fn add_to_rightmost(pair: PairTree, num: usize) -> PairTree {
            if num != 0 {
                match pair {
                    Num(current) => Num(current + num),
                    Pair(left, right) => PairTree::pair(*left, add_to_rightmost(*right, num)),
                }
            } else {
                pair
            }
        }

        match explode_subtree(0, self) {
            Left(SubTreeExplosionResult { new_subtree, .. }) => Left(new_subtree),
            Right(unchanged) => Right(unchanged),
        }
    }

    fn split(self) -> Either<PairTree, PairTree> {
        match self {
            Num(n) => {
                if n >= 10 {
                    Left(PairTree::pair(
                        Num((n as f64 / 2f64).floor() as usize),
                        Num((n as f64 / 2f64).ceil() as usize),
                    ))
                } else {
                    Right(Num(n))
                }
            }
            Pair(left, right) => match left.split() {
                Left(new_left) => Left(PairTree::pair(new_left, *right)),
                Right(unchanged_left) => match right.split() {
                    Left(new_right) => Left(PairTree::pair(unchanged_left, new_right)),
                    Right(unchanged_right) => {
                        Right(PairTree::pair(unchanged_left, unchanged_right))
                    }
                },
            },
        }
    }
}

impl Add for PairTree {
    type Output = PairTree;

    fn add(self, rhs: Self) -> Self::Output {
        PairTree::pair(self, rhs).reduce()
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
                        Num(2)
                    ),
                    Num(3)
                ),
                Num(4)
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
    fn biggest_pair_sum_test() {
        let i = parse(TEST_INPUT_MULTI).unwrap();
        let r = biggest_pair_sum(&i);
        assert_eq!(Some(3993), r)
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
                            PairTree::pair(Num(9), Num(6))
                        )
                    ),
                    PairTree::pair(
                        PairTree::pair(Num(4), PairTree::pair(Num(1), Num(2))),
                        PairTree::pair(PairTree::pair(Num(1), Num(4)), Num(2))
                    )
                ),
                PairTree::pair(
                    PairTree::pair(
                        PairTree::pair(Num(5), PairTree::pair(Num(2), Num(8))),
                        Num(4)
                    ),
                    PairTree::pair(
                        Num(5),
                        PairTree::pair(PairTree::pair(Num(9), Num(9)), Num(0))
                    )
                ),
                PairTree::pair(
                    Num(6),
                    PairTree::pair(
                        PairTree::pair(
                            PairTree::pair(Num(6), Num(2)),
                            PairTree::pair(Num(5), Num(6))
                        ),
                        PairTree::pair(
                            PairTree::pair(Num(7), Num(6)),
                            PairTree::pair(Num(4), Num(7))
                        )
                    )
                ),
                PairTree::pair(
                    PairTree::pair(
                        PairTree::pair(Num(6), PairTree::pair(Num(0), Num(7))),
                        PairTree::pair(Num(0), Num(9))
                    ),
                    PairTree::pair(
                        Num(4),
                        PairTree::pair(Num(9), PairTree::pair(Num(9), Num(0)))
                    )
                ),
                PairTree::pair(
                    PairTree::pair(
                        PairTree::pair(Num(7), PairTree::pair(Num(6), Num(4))),
                        PairTree::pair(Num(3), PairTree::pair(Num(1), Num(3)))
                    ),
                    PairTree::pair(
                        PairTree::pair(PairTree::pair(Num(5), Num(5)), Num(1)),
                        Num(9)
                    )
                ),
                PairTree::pair(
                    PairTree::pair(
                        Num(6),
                        PairTree::pair(
                            PairTree::pair(Num(7), Num(3)),
                            PairTree::pair(Num(3), Num(2))
                        )
                    ),
                    PairTree::pair(
                        PairTree::pair(
                            PairTree::pair(Num(3), Num(8)),
                            PairTree::pair(Num(5), Num(7))
                        ),
                        Num(4)
                    )
                ),
                PairTree::pair(
                    PairTree::pair(
                        PairTree::pair(
                            PairTree::pair(Num(5), Num(4)),
                            PairTree::pair(Num(7), Num(7))
                        ),
                        Num(8)
                    ),
                    PairTree::pair(PairTree::pair(Num(8), Num(3)), Num(8))
                ),
                PairTree::pair(
                    PairTree::pair(Num(9), Num(3)),
                    PairTree::pair(
                        PairTree::pair(Num(9), Num(9)),
                        PairTree::pair(Num(6), PairTree::pair(Num(4), Num(9)))
                    )
                ),
                PairTree::pair(
                    PairTree::pair(
                        Num(2),
                        PairTree::pair(PairTree::pair(Num(7), Num(7)), Num(7))
                    ),
                    PairTree::pair(
                        PairTree::pair(Num(5), Num(8)),
                        PairTree::pair(
                            PairTree::pair(Num(9), Num(3)),
                            PairTree::pair(Num(0), Num(2))
                        )
                    )
                ),
                PairTree::pair(
                    PairTree::pair(
                        PairTree::pair(PairTree::pair(Num(5), Num(2)), Num(5)),
                        PairTree::pair(Num(8), PairTree::pair(Num(3), Num(7)))
                    ),
                    PairTree::pair(
                        PairTree::pair(Num(5), PairTree::pair(Num(7), Num(5))),
                        PairTree::pair(Num(4), Num(4))
                    )
                )
            ]),
            i
        );
    }
}
