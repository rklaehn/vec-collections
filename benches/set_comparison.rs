use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::{BTreeSet, HashSet};
use vec_collections::*;

fn vs_create(n: u64) -> usize {
    let res: VecSet2<u64> = (0..n).collect();
    res.len()
}

fn vs_contains(x: &VecSet2<u64>, n: u64) -> usize {
    let mut res = 0;
    for e in 0..n {
        if x.contains(&e) {
            res += 1;
        }
    }
    res
}

fn bs_create(n: u64) -> usize {
    let res: BTreeSet<u64> = (0..n).collect();
    res.len()
}

fn bs_contains(x: &BTreeSet<u64>, n: u64) -> usize {
    let mut res = 0;
    for e in 0..n {
        if x.contains(&e) {
            res += 1;
        }
    }
    res
}

fn hs_create(n: u64) -> usize {
    let res: HashSet<u64> = (0..n).collect();
    res.len()
}

fn hs_contains(x: &HashSet<u64>, n: u64) -> usize {
    let mut res = 0;
    for e in 0..n {
        if x.contains(&e) {
            res += 1;
        }
    }
    res
}

pub fn creation(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "vs_create",
        |b, i| b.iter(|| vs_create(black_box(**i))),
        [100u64, 200].iter(),
    );
    c.bench_function_over_inputs(
        "bs_create",
        |b, i| b.iter(|| bs_create(black_box(**i))),
        [100u64, 200].iter(),
    );
    c.bench_function_over_inputs(
        "hs_create",
        |b, i| b.iter(|| hs_create(black_box(**i))),
        [100u64, 200].iter(),
    );
}
pub fn lookup(c: &mut Criterion) {
    let vs: VecSet2<u64> = (0..100).collect();
    let bs: BTreeSet<u64> = (0..100).collect();
    let hs: HashSet<u64> = (0..100).collect();
    c.bench_function("vs_lookup 100", |b| {
        b.iter(|| vs_contains(black_box(&vs), black_box(200)))
    });
    c.bench_function("bs_lookup 100", |b| {
        b.iter(|| bs_contains(black_box(&bs), black_box(200)))
    });
    c.bench_function("hs_lookup 100", |b| {
        b.iter(|| hs_contains(black_box(&hs), black_box(200)))
    });
}

criterion_group!(benches, creation, lookup);
criterion_main!(benches);
