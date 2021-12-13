use criterion::{black_box, criterion_group, criterion_main, Criterion};

use aoc_2021::*;

const DAY_12_INPUT: &str = include_str!("../data/day_12_input");

fn day_12_part_2(i: &day_12::Input) -> usize {
    day_12::generate_paths(i, true)
        .map(|v| v.len())
        .unwrap_or(0)
}

fn criterion_benchmark(c: &mut Criterion) {
    let day_12_input = day_12::parse(DAY_12_INPUT).expect("Should parse Day 12 fine");

    c.bench_function("Day 12 Part 2", |b| {
        b.iter(|| day_12_part_2(black_box(&day_12_input)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
