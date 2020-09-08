use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fnv::FnvHashSet;
use std::collections::{BTreeSet, HashSet};
use vec_collections::*;

fn vs_create(v: &Vec<u64>) -> usize {
    let res: VecSet2<u64> = v.iter().cloned().collect();
    res.len()
}

fn vs_contains(x: &VecSet2<u64>, values: &[u64]) -> usize {
    let mut res = 0;
    for e in values {
        if x.contains(e) {
            res += 1;
        }
    }
    res
}

fn bs_create(v: &Vec<u64>) -> usize {
    let res: BTreeSet<u64> = v.iter().cloned().collect();
    res.len()
}

fn bs_contains(x: &BTreeSet<u64>, values: &[u64]) -> usize {
    let mut res = 0;
    for e in values {
        if x.contains(e) {
            res += 1;
        }
    }
    res
}

fn hs_create(v: &Vec<u64>) -> usize {
    let res: HashSet<u64> = v.iter().cloned().collect();
    res.len()
}

fn hs_contains(x: &HashSet<u64>, values: &[u64]) -> usize {
    let mut res = 0;
    for e in values {
        if x.contains(e) {
            res += 1;
        }
    }
    res
}

fn fh_create(v: &Vec<u64>) -> usize {
    let res: FnvHashSet<u64> = v.iter().cloned().collect();
    res.len()
}

fn fh_contains(x: &FnvHashSet<u64>, values: &[u64]) -> usize {
    let mut res = 0;
    for e in values {
        if x.contains(e) {
            res += 1;
        }
    }
    res
}
use rand::seq::SliceRandom;
use rand::SeedableRng;

pub fn creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("creation");
    let mut rand = rand::rngs::StdRng::from_seed([0u8; 32]);
    for i in (100u64..=1000).step_by(100) {
        let mut values = (0..i).collect::<Vec<_>>();
        values.shuffle(&mut rand);

        group.bench_with_input(BenchmarkId::new("vs_create", i), &values, |b, values| {
            b.iter(|| vs_create(black_box(values)))
        });

        group.bench_with_input(BenchmarkId::new("bs_create", i), &values, |b, values| {
            b.iter(|| bs_create(black_box(values)))
        });

        group.bench_with_input(BenchmarkId::new("hs_create", i), &values, |b, values| {
            b.iter(|| hs_create(black_box(values)))
        });

        group.bench_with_input(BenchmarkId::new("fh_create", i), &values, |b, values| {
            b.iter(|| fh_create(black_box(values)))
        });
    }
}
pub fn lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup");
    let mut rand = rand::rngs::StdRng::from_seed([0u8; 32]);
    let lookup = 10;
    for i in (10u64..=100).step_by(10) {
        let mut values = (0..i).collect::<Vec<_>>();
        values.shuffle(&mut rand);

        let coll: VecSet2<u64> = values.iter().cloned().collect();
        group.bench_with_input(
            BenchmarkId::new("vs_lookup", i),
            &(coll, &values[0..10]),
            |b, coll| b.iter(|| vs_contains(black_box(&coll.0), &coll.1)),
        );

        let coll: BTreeSet<u64> = values.iter().cloned().collect();
        group.bench_with_input(
            BenchmarkId::new("bs_lookup", i),
            &(coll, &values[0..10]),
            |b, coll| b.iter(|| bs_contains(black_box(&coll.0), &coll.1)),
        );

        let coll: HashSet<u64> = values.iter().cloned().collect();
        group.bench_with_input(
            BenchmarkId::new("hs_lookup", i),
            &(coll, &values[0..10]),
            |b, coll| b.iter(|| hs_contains(black_box(&coll.0), &coll.1)),
        );

        let coll: FnvHashSet<u64> = values.iter().cloned().collect();
        group.bench_with_input(
            BenchmarkId::new("fh_lookup", i),
            &(coll, &values[0..10]),
            |b, coll| b.iter(|| fh_contains(black_box(&coll.0), &coll.1)),
        );
    }
}

criterion_group!(benches, creation, lookup);
criterion_main!(benches);
