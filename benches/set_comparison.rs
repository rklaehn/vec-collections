use std::collections::{BTreeSet, HashSet};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fnv::FnvHashSet;
use vec_collections::*;

fn vs_create(v: &[u32]) -> usize {
    let res: VecSet<[u32; 4]> = v.iter().cloned().collect();
    res.len()
}

fn vs_contains(x: &VecSet<[u32; 4]>, values: &[u32]) -> usize {
    let mut res = 0;
    for e in values {
        if x.contains(e) {
            res += 1;
        }
    }
    res
}

fn bs_create(v: &[u32]) -> usize {
    let res: BTreeSet<u32> = v.iter().cloned().collect();
    res.len()
}

fn bs_contains(x: &BTreeSet<u32>, values: &[u32]) -> usize {
    let mut res = 0;
    for e in values {
        if x.contains(e) {
            res += 1;
        }
    }
    res
}

fn hs_create(v: &[u32]) -> usize {
    let res: HashSet<u32> = v.iter().cloned().collect();
    res.len()
}

fn hs_contains(x: &HashSet<u32>, values: &[u32]) -> usize {
    let mut res = 0;
    for e in values {
        if x.contains(e) {
            res += 1;
        }
    }
    res
}

fn fh_create(v: &[u32]) -> usize {
    let res: FnvHashSet<u32> = v.iter().cloned().collect();
    res.len()
}

fn fh_contains(x: &FnvHashSet<u32>, values: &[u32]) -> usize {
    let mut res = 0;
    for e in values {
        if x.contains(e) {
            res += 1;
        }
    }
    res
}
use rand::{seq::SliceRandom, SeedableRng};

fn creation_bench(c: &mut Criterion, title: &str, range: impl Iterator<Item = u32>) {
    let mut group = c.benchmark_group(format!("Creation {}", title));
    let mut rand = rand::rngs::StdRng::from_seed([0u8; 32]);
    for i in range {
        let mut values = (0..i).collect::<Vec<_>>();
        values.shuffle(&mut rand);

        group.bench_with_input(
            BenchmarkId::new("VecSet<[u32; 4]> create", i),
            &values,
            |b, values| b.iter(|| vs_create(black_box(values))),
        );

        group.bench_with_input(
            BenchmarkId::new("BTreeSet<u32> create", i),
            &values,
            |b, values| b.iter(|| bs_create(black_box(values))),
        );

        group.bench_with_input(
            BenchmarkId::new("HashSet<u32> creaete", i),
            &values,
            |b, values| b.iter(|| hs_create(black_box(values))),
        );

        group.bench_with_input(
            BenchmarkId::new("FnvHashSet<u32> creaete", i),
            &values,
            |b, values| b.iter(|| fh_create(black_box(values))),
        );
    }
}
pub fn creation_medium(c: &mut Criterion) {
    creation_bench(c, "medium", (10..=100).step_by(10))
}
pub fn creation_small(c: &mut Criterion) {
    creation_bench(c, "small", 1..=10)
}
pub fn lookup_bench(c: &mut Criterion, title: &str, range: impl Iterator<Item = u32>) {
    let mut group = c.benchmark_group(format!("Lookup {}", title));
    let mut rand = rand::rngs::StdRng::from_seed([0u8; 32]);
    let lookup = 10;
    for i in range {
        let mut values = (0..i).collect::<Vec<_>>();
        values.shuffle(&mut rand);
        let lookup = (0..lookup)
            .map(|x| values[x % values.len()])
            .collect::<Vec<_>>();

        let coll: VecSet<[u32; 4]> = values.iter().cloned().collect();
        group.bench_with_input(
            BenchmarkId::new("VecSet<[u32; 4]> lookup 10", i),
            &(coll, &lookup),
            |b, coll| b.iter(|| vs_contains(black_box(&coll.0), coll.1)),
        );

        let coll: BTreeSet<u32> = values.iter().cloned().collect();
        group.bench_with_input(
            BenchmarkId::new("BTreeSet<u32> lookup 10", i),
            &(coll, &lookup),
            |b, coll| b.iter(|| bs_contains(black_box(&coll.0), coll.1)),
        );

        let coll: HashSet<u32> = values.iter().cloned().collect();
        group.bench_with_input(
            BenchmarkId::new("HashSet<u32> lookup 10", i),
            &(coll, &lookup),
            |b, coll| b.iter(|| hs_contains(black_box(&coll.0), coll.1)),
        );

        let coll: FnvHashSet<u32> = values.iter().cloned().collect();
        group.bench_with_input(
            BenchmarkId::new("FnvHashSet<u32> lookup 10", i),
            &(coll, &lookup),
            |b, coll| b.iter(|| fh_contains(black_box(&coll.0), coll.1)),
        );
    }
}
pub fn lookup_medium(c: &mut Criterion) {
    lookup_bench(c, "medium", (10..=100).step_by(10))
}
pub fn lookup_small(c: &mut Criterion) {
    lookup_bench(c, "small", 1..=10)
}

criterion_group!(
    benches,
    creation_small,
    lookup_small,
    creation_medium,
    lookup_medium
);
criterion_main!(benches);
