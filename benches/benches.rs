use criterion::{black_box, criterion_group, criterion_main, Criterion};

use aoc_2021::*;

fn day_12_part_2(i: &day_12::Input) -> usize {
    day_12::generate_paths(i, true)
        .map(|v| v.len())
        .unwrap_or(0)
}

fn day_13_part_2(i: &day_13::Input) -> usize {
    i.fold_all().dots.len()
}

fn day_14_part_2(i: &day_14::Input) -> usize {
    i.max_minus_least_after_steps(40).unwrap_or(0)
}

fn day_15_part_2(i: &day_15::Input) -> usize {
    i.lowest_risk_to_end().unwrap_or(0)
}

fn day_16_decode(i: &day_16::Input) -> usize {
    i.packet.decode().map(|r| r.packet_version()).unwrap_or(0)
}

fn day_16_part_1(i: &day_16::DecodedPacket) -> usize {
    i.version_sum()
}

fn day_16_part_2(i: &day_16::DecodedPacket) -> usize {
    i.run().unwrap_or(0)
}

fn criterion_benchmark(c: &mut Criterion) {
    let day_12_input = day_12::parse(day_12::INPUT).expect("Should parse Day 12 fine");
    let day_13_input = day_13::parse(day_13::INPUT).expect("Should parse Day 13 fine");
    let day_14_input = day_14::parse(day_14::INPUT).expect("Should parse Day 14 fine");
    let day_15_input = day_15::parse(day_15::INPUT)
        .expect("Should parse Day 15 fine")
        .expand(5);
    let day_16_input = day_16::parse(day_16::INPUT).expect("Should parse Day 16 fine");
    let day_16_input_decoded = day_16_input
        .packet
        .decode()
        .expect("Should decode Day 16 input fine");

    c.bench_function("Day 12 Part 2", |b| {
        b.iter(|| day_12_part_2(black_box(&day_12_input)))
    });

    c.bench_function("Day 13 Part 2", |b| {
        b.iter(|| day_13_part_2(black_box(&day_13_input)))
    });

    c.bench_function("Day 14 Part 2", |b| {
        b.iter(|| day_14_part_2(black_box(&day_14_input)))
    });

    c.bench_function("Day 15 Part 2", |b| {
        b.iter(|| day_15_part_2(black_box(&day_15_input)))
    });

    c.bench_function("Day 16 Decoding", |b| {
        b.iter(|| day_16_decode(black_box(&day_16_input)))
    });

    c.bench_function("Day 16 Part 1", |b| {
        b.iter(|| day_16_part_1(black_box(&day_16_input_decoded)))
    });

    c.bench_function("Day 16 Part 2", |b| {
        b.iter(|| day_16_part_2(black_box(&day_16_input_decoded)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
