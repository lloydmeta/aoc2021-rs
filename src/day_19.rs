use std::collections::{HashSet, VecDeque};

use anyhow::{Context, Result};
use combine::parser::char::*;
use combine::*;
use itertools::Itertools;

use crate::common::*;

const MIN_OVERLAPS: usize = 12;
pub const INPUT: &str = include_str!("../data/day_19_input");

pub fn run() -> Result<()> {
    println!("*** Day 19: Beacon Scanner ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let map = input.build_map().context("Failed to build map")?;
    let sol_1 = map.beacons.len();
    println!("Solution 1: {:?}", sol_1);
    let sol_2 = map.max_distance_between_scanners();
    println!("Solution 2: {:?}", sol_2);

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub struct Input(Vec<Scanner>);

impl Input {
    pub fn build_map(&self) -> Option<Map> {
        if let Some(first) = self.0.first() {
            let mut map = Map {
                scanners: vec![Point { x: 0, y: 0, z: 0 }],
                beacons: first.beacons.clone(),
            };
            let mut unmapped_scanner_indices = (1..self.0.len()).collect::<VecDeque<_>>();
            while let Some(scanner_idx) = unmapped_scanner_indices.pop_front() {
                if let Some(scanner) = self.0.get(scanner_idx) {
                    if !scanner.merge_into(&mut map) {
                        unmapped_scanner_indices.push_back(scanner_idx); // Revisit
                    }
                }
            }
            Some(map)
        } else {
            None
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Map {
    pub scanners: Vec<Point>,
    beacons: HashSet<Point>,
}

impl Map {
    fn max_distance_between_scanners(&self) -> Option<usize> {
        self.scanners
            .iter()
            .tuple_combinations()
            .map(|(s1, s2)| {
                ((s1.x - s2.x).abs() + (s1.y - s2.y).abs() + (s1.z - s2.z).abs()) as usize
            })
            .max()
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
pub struct Point {
    x: isize,
    y: isize,
    z: isize,
}

impl Point {
    /// Returns different perspectives of self given a perspective index from 0-23.
    ///
    /// If the idx provide is higher than that range, just returns self.
    ///
    /// Hard-coded Because I'm an idiot and don't want to do matrix stuff or pull in a lib
    #[allow(clippy::redundant_field_names)]
    fn perspective_at_idx(&self, idx: usize) -> Point {
        let Point { x, y, z } = *self;
        match idx {
            0 => Point { x: x, y: y, z: z },
            1 => Point { x: x, y: z, z: -y },
            2 => Point { x: x, y: -y, z: -z },
            3 => Point { x: x, y: -z, z: y },
            4 => Point { x: y, y: x, z: -z },
            5 => Point { x: y, y: z, z: x },
            6 => Point { x: y, y: -x, z: z },
            7 => Point { x: y, y: -z, z: -x },
            8 => Point { x: z, y: x, z: y },
            9 => Point { x: z, y: y, z: -x },
            10 => Point { x: z, y: -x, z: -y },
            11 => Point { x: z, y: -y, z: x },
            12 => Point { x: -x, y: y, z: -z },
            13 => Point { x: -x, y: z, z: y },
            14 => Point { x: -x, y: -y, z: z },
            15 => Point {
                x: -x,
                y: -z,
                z: -y,
            },
            16 => Point { x: -y, y: x, z: z },
            17 => Point { x: -y, y: z, z: -x },
            18 => Point {
                x: -y,
                y: -x,
                z: -z,
            },
            19 => Point { x: -y, y: -z, z: x },
            20 => Point { x: -z, y: x, z: -y },
            21 => Point { x: -z, y: y, z: x },
            22 => Point { x: -z, y: -x, z: y },
            23 => Point {
                x: -z,
                y: -y,
                z: -x,
            },
            _ => *self,
        }
    }

    /// Relativises the given point against self
    fn relativise(&self, p: &Point) -> Point {
        Point {
            x: p.x - self.x,
            y: p.y - self.y,
            z: p.z - self.z,
        }
    }

    /// Anchors the give pont against self
    fn anchor(&self, p: &Point) -> Point {
        Point {
            x: p.x + self.x,
            y: p.y + self.y,
            z: p.z + self.z,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Scanner {
    idx: usize,
    beacons: HashSet<Point>,
}

impl Scanner {
    fn different_perspectives(&self) -> impl Iterator<Item = Scanner> + '_ {
        (0..24).map(move |p_idx| {
            let beacons_at_perspective = self
                .beacons
                .iter()
                .map(|p| p.perspective_at_idx(p_idx))
                .collect();
            Scanner {
                idx: self.idx,
                beacons: beacons_at_perspective,
            }
        })
    }

    /// Attempts to merge this [Scanner] into the provided Map
    ///
    /// Returns false if there were not enough overlaps found despite rotating between all
    /// perspectives of self and relative positions, true otherwise.
    fn merge_into(&self, map: &mut Map) -> bool {
        for self_perspective in self.different_perspectives() {
            for map_anchor_beacon in map.beacons.iter() {
                let map_beacons_relative_to_map_anchor_beacon = map
                    .beacons
                    .iter()
                    .map(|p| map_anchor_beacon.relativise(p))
                    .collect::<HashSet<_>>();
                for self_anchor_beacon in &self_perspective.beacons {
                    let self_beacons_relative_to_self_anchor_beacon = self_perspective
                        .beacons
                        .iter()
                        .map(|p| self_anchor_beacon.relativise(p))
                        .collect::<HashSet<_>>();
                    let overlaps = map_beacons_relative_to_map_anchor_beacon
                        .intersection(&self_beacons_relative_to_self_anchor_beacon)
                        .count();
                    if overlaps >= MIN_OVERLAPS {
                        let self_perspective_relative_to_map_anchor =
                            self_anchor_beacon.relativise(map_anchor_beacon);
                        let self_beacons_anchored_to_map_anchor =
                            self_beacons_relative_to_self_anchor_beacon
                                .iter()
                                .map(|p| map_anchor_beacon.anchor(p))
                                .collect::<Vec<_>>();
                        for beacon in self_beacons_anchored_to_map_anchor {
                            map.beacons.insert(beacon);
                        }
                        map.scanners.push(self_perspective_relative_to_map_anchor);
                        return true;
                    }
                }
            }
        }
        false
    }
}

// Ugh, the double newline separator still messes with me...
// TODO: try with pure Combine, again (https://gitter.im/Marwes/combine?at=5fde845622f12e449bfe9459)
pub fn parse(s: &str) -> Result<Input> {
    let split_by_newline = s.trim().split("\n\n"); // ugh.... really need to figure out how to do this with just combine...

    let scanners = split_by_newline
        .into_iter()
        .filter_map(|scanner_str| {
            let scanner_header_parser = string("--- scanner ")
                .with(usize_parser())
                .skip(string(" ---"));
            let point_parser = isize_parser()
                .skip(char(','))
                .and(isize_parser())
                .skip(char(','))
                .and(isize_parser())
                .map(|((x, y), z)| Point { x, y, z });

            let mut parser = scanner_header_parser
                .skip(newline())
                .and(sep_by1(point_parser, newline()))
                .map(|(idx, beacons)| Scanner { idx, beacons });

            let (scanner, _) = parser.easy_parse(scanner_str.trim()).ok()?;
            Some(scanner)
        })
        .collect();

    Ok(Input(scanners))
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use super::*;

    static TEST_INPUT: &str = "--- scanner 0 ---
404,-588,-901
528,-643,409
-838,591,734
390,-675,-793
-537,-823,-458
-485,-357,347
-345,-311,381
-661,-816,-575
-876,649,763
-618,-824,-621
553,345,-567
474,580,667
-447,-329,318
-584,868,-557
544,-627,-890
564,392,-477
455,729,728
-892,524,684
-689,845,-530
423,-701,434
7,-33,-71
630,319,-379
443,580,662
-789,900,-551
459,-707,401

--- scanner 1 ---
686,422,578
605,423,415
515,917,-361
-336,658,858
95,138,22
-476,619,847
-340,-569,-846
567,-361,727
-460,603,-452
669,-402,600
729,430,532
-500,-761,534
-322,571,750
-466,-666,-811
-429,-592,574
-355,545,-477
703,-491,-529
-328,-685,520
413,935,-424
-391,539,-444
586,-435,557
-364,-763,-893
807,-499,-711
755,-354,-619
553,889,-390

--- scanner 2 ---
649,640,665
682,-795,504
-784,533,-524
-644,584,-595
-588,-843,648
-30,6,44
-674,560,763
500,723,-460
609,671,-379
-555,-800,653
-675,-892,-343
697,-426,-610
578,704,681
493,664,-388
-671,-858,530
-667,343,800
571,-461,-707
-138,-166,112
-889,563,-600
646,-828,498
640,759,510
-630,509,768
-681,-892,-333
673,-379,-804
-742,-814,-386
577,-820,562

--- scanner 3 ---
-589,542,597
605,-692,669
-500,565,-823
-660,373,557
-458,-679,-417
-488,449,543
-626,468,-788
338,-750,-386
528,-832,-391
562,-778,733
-938,-730,414
543,643,-506
-524,371,-870
407,773,750
-104,29,83
378,-903,-323
-778,-728,485
426,699,580
-438,-605,-362
-469,-447,-387
509,732,623
647,635,-688
-868,-804,481
614,-800,639
595,780,-596

--- scanner 4 ---
727,592,562
-293,-554,779
441,611,-461
-714,465,-776
-743,427,-804
-660,-479,-426
832,-632,460
927,-485,-438
408,393,-506
466,436,-512
110,16,151
-258,-428,682
-393,719,612
-211,-452,876
808,-476,-593
-575,615,604
-485,667,467
-680,325,-822
-627,-443,-432
872,-547,-609
833,512,582
807,604,487
839,-516,451
891,-625,532
-652,-548,-490
30,-46,-14";

    #[test]
    fn different_perspectives_test() {
        let p = Point { x: 1, y: 2, z: 3 };

        let r = (0..24).map(|idx| p.perspective_at_idx(idx));
        let distincts: Vec<_> = r.unique().collect();
        assert_eq!(24, distincts.len())
    }

    #[test]
    fn map_max_distance_between_scanners_test() {
        let i = parse(TEST_INPUT).unwrap();
        let r = i.build_map().unwrap();

        assert_eq!(Some(3621), r.max_distance_between_scanners())
    }

    #[test]
    fn build_map_real_test() {
        let i = parse(INPUT).unwrap();
        let r = i.build_map().unwrap();

        assert_eq!(438, r.beacons.len());
        assert_eq!(Some(11985), r.max_distance_between_scanners());
    }

    #[test]
    fn build_map_test() {
        let i = parse(TEST_INPUT).unwrap();
        let r = i.build_map().unwrap();

        let expected_beacons = HashSet::from_iter([
            Point {
                x: -892,
                y: 524,
                z: 684,
            },
            Point {
                x: -876,
                y: 649,
                z: 763,
            },
            Point {
                x: -838,
                y: 591,
                z: 734,
            },
            Point {
                x: -789,
                y: 900,
                z: -551,
            },
            Point {
                x: -739,
                y: -1745,
                z: 668,
            },
            Point {
                x: -706,
                y: -3180,
                z: -659,
            },
            Point {
                x: -697,
                y: -3072,
                z: -689,
            },
            Point {
                x: -689,
                y: 845,
                z: -530,
            },
            Point {
                x: -687,
                y: -1600,
                z: 576,
            },
            Point {
                x: -661,
                y: -816,
                z: -575,
            },
            Point {
                x: -654,
                y: -3158,
                z: -753,
            },
            Point {
                x: -635,
                y: -1737,
                z: 486,
            },
            Point {
                x: -631,
                y: -672,
                z: 1502,
            },
            Point {
                x: -624,
                y: -1620,
                z: 1868,
            },
            Point {
                x: -620,
                y: -3212,
                z: 371,
            },
            Point {
                x: -618,
                y: -824,
                z: -621,
            },
            Point {
                x: -612,
                y: -1695,
                z: 1788,
            },
            Point {
                x: -601,
                y: -1648,
                z: -643,
            },
            Point {
                x: -584,
                y: 868,
                z: -557,
            },
            Point {
                x: -537,
                y: -823,
                z: -458,
            },
            Point {
                x: -532,
                y: -1715,
                z: 1894,
            },
            Point {
                x: -518,
                y: -1681,
                z: -600,
            },
            Point {
                x: -499,
                y: -1607,
                z: -770,
            },
            Point {
                x: -485,
                y: -357,
                z: 347,
            },
            Point {
                x: -470,
                y: -3283,
                z: 303,
            },
            Point {
                x: -456,
                y: -621,
                z: 1527,
            },
            Point {
                x: -447,
                y: -329,
                z: 318,
            },
            Point {
                x: -430,
                y: -3130,
                z: 366,
            },
            Point {
                x: -413,
                y: -627,
                z: 1469,
            },
            Point {
                x: -345,
                y: -311,
                z: 381,
            },
            Point {
                x: -36,
                y: -1284,
                z: 1171,
            },
            Point {
                x: -27,
                y: -1108,
                z: -65,
            },
            Point {
                x: 7,
                y: -33,
                z: -71,
            },
            Point {
                x: 12,
                y: -2351,
                z: -103,
            },
            Point {
                x: 26,
                y: -1119,
                z: 1091,
            },
            Point {
                x: 346,
                y: -2985,
                z: 342,
            },
            Point {
                x: 366,
                y: -3059,
                z: 397,
            },
            Point {
                x: 377,
                y: -2827,
                z: 367,
            },
            Point {
                x: 390,
                y: -675,
                z: -793,
            },
            Point {
                x: 396,
                y: -1931,
                z: -563,
            },
            Point {
                x: 404,
                y: -588,
                z: -901,
            },
            Point {
                x: 408,
                y: -1815,
                z: 803,
            },
            Point {
                x: 423,
                y: -701,
                z: 434,
            },
            Point {
                x: 432,
                y: -2009,
                z: 850,
            },
            Point {
                x: 443,
                y: 580,
                z: 662,
            },
            Point {
                x: 455,
                y: 729,
                z: 728,
            },
            Point {
                x: 456,
                y: -540,
                z: 1869,
            },
            Point {
                x: 459,
                y: -707,
                z: 401,
            },
            Point {
                x: 465,
                y: -695,
                z: 1988,
            },
            Point {
                x: 474,
                y: 580,
                z: 667,
            },
            Point {
                x: 496,
                y: -1584,
                z: 1900,
            },
            Point {
                x: 497,
                y: -1838,
                z: -617,
            },
            Point {
                x: 527,
                y: -524,
                z: 1933,
            },
            Point {
                x: 528,
                y: -643,
                z: 409,
            },
            Point {
                x: 534,
                y: -1912,
                z: 768,
            },
            Point {
                x: 544,
                y: -627,
                z: -890,
            },
            Point {
                x: 553,
                y: 345,
                z: -567,
            },
            Point {
                x: 564,
                y: 392,
                z: -477,
            },
            Point {
                x: 568,
                y: -2007,
                z: -577,
            },
            Point {
                x: 605,
                y: -1665,
                z: 1952,
            },
            Point {
                x: 612,
                y: -1593,
                z: 1893,
            },
            Point {
                x: 630,
                y: 319,
                z: -379,
            },
            Point {
                x: 686,
                y: -3108,
                z: -505,
            },
            Point {
                x: 776,
                y: -3184,
                z: -501,
            },
            Point {
                x: 846,
                y: -3110,
                z: -434,
            },
            Point {
                x: 1135,
                y: -1161,
                z: 1235,
            },
            Point {
                x: 1243,
                y: -1093,
                z: 1063,
            },
            Point {
                x: 1660,
                y: -552,
                z: 429,
            },
            Point {
                x: 1693,
                y: -557,
                z: 386,
            },
            Point {
                x: 1735,
                y: -437,
                z: 1738,
            },
            Point {
                x: 1749,
                y: -1800,
                z: 1813,
            },
            Point {
                x: 1772,
                y: -405,
                z: 1572,
            },
            Point {
                x: 1776,
                y: -675,
                z: 371,
            },
            Point {
                x: 1779,
                y: -442,
                z: 1789,
            },
            Point {
                x: 1780,
                y: -1548,
                z: 337,
            },
            Point {
                x: 1786,
                y: -1538,
                z: 337,
            },
            Point {
                x: 1847,
                y: -1591,
                z: 415,
            },
            Point {
                x: 1889,
                y: -1729,
                z: 1762,
            },
            Point {
                x: 1994,
                y: -1805,
                z: 1792,
            },
        ]);

        // assert_eq!(5, r.scanners.len());
        assert_eq!(5, r.scanners.len());
        assert_eq!(expected_beacons, r.beacons);
    }

    #[test]
    fn different_scanner_perspectives_test() {
        let input = parse(
            "--- scanner 0 ---
-1,-1,1
-2,-2,2
-3,-3,3
-2,-3,1
5,6,-4
8,0,7",
        )
        .unwrap();
        let first_scanner = input.0.get(0).unwrap();
        let views_of_first_scanner = first_scanner.different_perspectives().collect::<Vec<_>>();

        let i = parse(
            "--- scanner 0 ---
1,-1,1
2,-2,2
3,-3,3
2,-1,3
-5,4,-6
-8,-7,0",
        )
        .unwrap();
        let one = i.0.get(0).unwrap();

        assert!(views_of_first_scanner.contains(one));

        let i = parse(
            "--- scanner 0 ---
-1,-1,-1
-2,-2,-2
-3,-3,-3
-1,-3,-2
4,6,5
-7,0,8",
        )
        .unwrap();
        let one = i.0.get(0).unwrap();

        assert!(views_of_first_scanner.contains(one));

        let i = parse(
            "--- scanner 0 ---
1,1,-1
2,2,-2
3,3,-3
1,3,-2
-4,-6,5
7,0,8",
        )
        .unwrap();
        let one = i.0.get(0).unwrap();

        assert!(views_of_first_scanner.contains(one));
        let i = parse(
            "--- scanner 0 ---
1,1,1
2,2,2
3,3,3
3,1,2
-6,-4,-5
0,7,-8",
        )
        .unwrap();
        let one = i.0.get(0).unwrap();

        assert!(views_of_first_scanner.contains(one));
    }

    #[test]
    fn parse_test() {
        let i = parse(TEST_INPUT).unwrap();
        let expected = Input(vec![
            Scanner {
                idx: 0,
                beacons: HashSet::from_iter([
                    Point {
                        x: 404,
                        y: -588,
                        z: -901,
                    },
                    Point {
                        x: 528,
                        y: -643,
                        z: 409,
                    },
                    Point {
                        x: -838,
                        y: 591,
                        z: 734,
                    },
                    Point {
                        x: 390,
                        y: -675,
                        z: -793,
                    },
                    Point {
                        x: -537,
                        y: -823,
                        z: -458,
                    },
                    Point {
                        x: -485,
                        y: -357,
                        z: 347,
                    },
                    Point {
                        x: -345,
                        y: -311,
                        z: 381,
                    },
                    Point {
                        x: -661,
                        y: -816,
                        z: -575,
                    },
                    Point {
                        x: -876,
                        y: 649,
                        z: 763,
                    },
                    Point {
                        x: -618,
                        y: -824,
                        z: -621,
                    },
                    Point {
                        x: 553,
                        y: 345,
                        z: -567,
                    },
                    Point {
                        x: 474,
                        y: 580,
                        z: 667,
                    },
                    Point {
                        x: -447,
                        y: -329,
                        z: 318,
                    },
                    Point {
                        x: -584,
                        y: 868,
                        z: -557,
                    },
                    Point {
                        x: 544,
                        y: -627,
                        z: -890,
                    },
                    Point {
                        x: 564,
                        y: 392,
                        z: -477,
                    },
                    Point {
                        x: 455,
                        y: 729,
                        z: 728,
                    },
                    Point {
                        x: -892,
                        y: 524,
                        z: 684,
                    },
                    Point {
                        x: -689,
                        y: 845,
                        z: -530,
                    },
                    Point {
                        x: 423,
                        y: -701,
                        z: 434,
                    },
                    Point {
                        x: 7,
                        y: -33,
                        z: -71,
                    },
                    Point {
                        x: 630,
                        y: 319,
                        z: -379,
                    },
                    Point {
                        x: 443,
                        y: 580,
                        z: 662,
                    },
                    Point {
                        x: -789,
                        y: 900,
                        z: -551,
                    },
                    Point {
                        x: 459,
                        y: -707,
                        z: 401,
                    },
                ]),
            },
            Scanner {
                idx: 1,
                beacons: HashSet::from_iter([
                    Point {
                        x: 686,
                        y: 422,
                        z: 578,
                    },
                    Point {
                        x: 605,
                        y: 423,
                        z: 415,
                    },
                    Point {
                        x: 515,
                        y: 917,
                        z: -361,
                    },
                    Point {
                        x: -336,
                        y: 658,
                        z: 858,
                    },
                    Point {
                        x: 95,
                        y: 138,
                        z: 22,
                    },
                    Point {
                        x: -476,
                        y: 619,
                        z: 847,
                    },
                    Point {
                        x: -340,
                        y: -569,
                        z: -846,
                    },
                    Point {
                        x: 567,
                        y: -361,
                        z: 727,
                    },
                    Point {
                        x: -460,
                        y: 603,
                        z: -452,
                    },
                    Point {
                        x: 669,
                        y: -402,
                        z: 600,
                    },
                    Point {
                        x: 729,
                        y: 430,
                        z: 532,
                    },
                    Point {
                        x: -500,
                        y: -761,
                        z: 534,
                    },
                    Point {
                        x: -322,
                        y: 571,
                        z: 750,
                    },
                    Point {
                        x: -466,
                        y: -666,
                        z: -811,
                    },
                    Point {
                        x: -429,
                        y: -592,
                        z: 574,
                    },
                    Point {
                        x: -355,
                        y: 545,
                        z: -477,
                    },
                    Point {
                        x: 703,
                        y: -491,
                        z: -529,
                    },
                    Point {
                        x: -328,
                        y: -685,
                        z: 520,
                    },
                    Point {
                        x: 413,
                        y: 935,
                        z: -424,
                    },
                    Point {
                        x: -391,
                        y: 539,
                        z: -444,
                    },
                    Point {
                        x: 586,
                        y: -435,
                        z: 557,
                    },
                    Point {
                        x: -364,
                        y: -763,
                        z: -893,
                    },
                    Point {
                        x: 807,
                        y: -499,
                        z: -711,
                    },
                    Point {
                        x: 755,
                        y: -354,
                        z: -619,
                    },
                    Point {
                        x: 553,
                        y: 889,
                        z: -390,
                    },
                ]),
            },
            Scanner {
                idx: 2,
                beacons: HashSet::from_iter([
                    Point {
                        x: 649,
                        y: 640,
                        z: 665,
                    },
                    Point {
                        x: 682,
                        y: -795,
                        z: 504,
                    },
                    Point {
                        x: -784,
                        y: 533,
                        z: -524,
                    },
                    Point {
                        x: -644,
                        y: 584,
                        z: -595,
                    },
                    Point {
                        x: -588,
                        y: -843,
                        z: 648,
                    },
                    Point {
                        x: -30,
                        y: 6,
                        z: 44,
                    },
                    Point {
                        x: -674,
                        y: 560,
                        z: 763,
                    },
                    Point {
                        x: 500,
                        y: 723,
                        z: -460,
                    },
                    Point {
                        x: 609,
                        y: 671,
                        z: -379,
                    },
                    Point {
                        x: -555,
                        y: -800,
                        z: 653,
                    },
                    Point {
                        x: -675,
                        y: -892,
                        z: -343,
                    },
                    Point {
                        x: 697,
                        y: -426,
                        z: -610,
                    },
                    Point {
                        x: 578,
                        y: 704,
                        z: 681,
                    },
                    Point {
                        x: 493,
                        y: 664,
                        z: -388,
                    },
                    Point {
                        x: -671,
                        y: -858,
                        z: 530,
                    },
                    Point {
                        x: -667,
                        y: 343,
                        z: 800,
                    },
                    Point {
                        x: 571,
                        y: -461,
                        z: -707,
                    },
                    Point {
                        x: -138,
                        y: -166,
                        z: 112,
                    },
                    Point {
                        x: -889,
                        y: 563,
                        z: -600,
                    },
                    Point {
                        x: 646,
                        y: -828,
                        z: 498,
                    },
                    Point {
                        x: 640,
                        y: 759,
                        z: 510,
                    },
                    Point {
                        x: -630,
                        y: 509,
                        z: 768,
                    },
                    Point {
                        x: -681,
                        y: -892,
                        z: -333,
                    },
                    Point {
                        x: 673,
                        y: -379,
                        z: -804,
                    },
                    Point {
                        x: -742,
                        y: -814,
                        z: -386,
                    },
                    Point {
                        x: 577,
                        y: -820,
                        z: 562,
                    },
                ]),
            },
            Scanner {
                idx: 3,
                beacons: HashSet::from_iter([
                    Point {
                        x: -589,
                        y: 542,
                        z: 597,
                    },
                    Point {
                        x: 605,
                        y: -692,
                        z: 669,
                    },
                    Point {
                        x: -500,
                        y: 565,
                        z: -823,
                    },
                    Point {
                        x: -660,
                        y: 373,
                        z: 557,
                    },
                    Point {
                        x: -458,
                        y: -679,
                        z: -417,
                    },
                    Point {
                        x: -488,
                        y: 449,
                        z: 543,
                    },
                    Point {
                        x: -626,
                        y: 468,
                        z: -788,
                    },
                    Point {
                        x: 338,
                        y: -750,
                        z: -386,
                    },
                    Point {
                        x: 528,
                        y: -832,
                        z: -391,
                    },
                    Point {
                        x: 562,
                        y: -778,
                        z: 733,
                    },
                    Point {
                        x: -938,
                        y: -730,
                        z: 414,
                    },
                    Point {
                        x: 543,
                        y: 643,
                        z: -506,
                    },
                    Point {
                        x: -524,
                        y: 371,
                        z: -870,
                    },
                    Point {
                        x: 407,
                        y: 773,
                        z: 750,
                    },
                    Point {
                        x: -104,
                        y: 29,
                        z: 83,
                    },
                    Point {
                        x: 378,
                        y: -903,
                        z: -323,
                    },
                    Point {
                        x: -778,
                        y: -728,
                        z: 485,
                    },
                    Point {
                        x: 426,
                        y: 699,
                        z: 580,
                    },
                    Point {
                        x: -438,
                        y: -605,
                        z: -362,
                    },
                    Point {
                        x: -469,
                        y: -447,
                        z: -387,
                    },
                    Point {
                        x: 509,
                        y: 732,
                        z: 623,
                    },
                    Point {
                        x: 647,
                        y: 635,
                        z: -688,
                    },
                    Point {
                        x: -868,
                        y: -804,
                        z: 481,
                    },
                    Point {
                        x: 614,
                        y: -800,
                        z: 639,
                    },
                    Point {
                        x: 595,
                        y: 780,
                        z: -596,
                    },
                ]),
            },
            Scanner {
                idx: 4,
                beacons: HashSet::from_iter([
                    Point {
                        x: 727,
                        y: 592,
                        z: 562,
                    },
                    Point {
                        x: -293,
                        y: -554,
                        z: 779,
                    },
                    Point {
                        x: 441,
                        y: 611,
                        z: -461,
                    },
                    Point {
                        x: -714,
                        y: 465,
                        z: -776,
                    },
                    Point {
                        x: -743,
                        y: 427,
                        z: -804,
                    },
                    Point {
                        x: -660,
                        y: -479,
                        z: -426,
                    },
                    Point {
                        x: 832,
                        y: -632,
                        z: 460,
                    },
                    Point {
                        x: 927,
                        y: -485,
                        z: -438,
                    },
                    Point {
                        x: 408,
                        y: 393,
                        z: -506,
                    },
                    Point {
                        x: 466,
                        y: 436,
                        z: -512,
                    },
                    Point {
                        x: 110,
                        y: 16,
                        z: 151,
                    },
                    Point {
                        x: -258,
                        y: -428,
                        z: 682,
                    },
                    Point {
                        x: -393,
                        y: 719,
                        z: 612,
                    },
                    Point {
                        x: -211,
                        y: -452,
                        z: 876,
                    },
                    Point {
                        x: 808,
                        y: -476,
                        z: -593,
                    },
                    Point {
                        x: -575,
                        y: 615,
                        z: 604,
                    },
                    Point {
                        x: -485,
                        y: 667,
                        z: 467,
                    },
                    Point {
                        x: -680,
                        y: 325,
                        z: -822,
                    },
                    Point {
                        x: -627,
                        y: -443,
                        z: -432,
                    },
                    Point {
                        x: 872,
                        y: -547,
                        z: -609,
                    },
                    Point {
                        x: 833,
                        y: 512,
                        z: 582,
                    },
                    Point {
                        x: 807,
                        y: 604,
                        z: 487,
                    },
                    Point {
                        x: 839,
                        y: -516,
                        z: 451,
                    },
                    Point {
                        x: 891,
                        y: -625,
                        z: 532,
                    },
                    Point {
                        x: -652,
                        y: -548,
                        z: -490,
                    },
                    Point {
                        x: 30,
                        y: -46,
                        z: -14,
                    },
                ]),
            },
        ]);
        assert_eq!(expected, i);
    }
}
