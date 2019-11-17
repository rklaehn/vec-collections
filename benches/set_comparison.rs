use vec_collections::*;
use std::collections::{BTreeSet, HashSet};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn vec_set_create(n: u64) -> usize {
    let res: VecSet<u64> = (0..n).collect();
    res.len()
}

fn btree_set_create(n: u64) -> usize {
    let res: BTreeSet<u64> = (0..n).collect();
    res.len()
}

fn hash_set_create(n: u64) -> usize {
    let res: HashSet<u64> = (0..n).collect();
    res.len()
}

pub fn creation(c: &mut Criterion) {
    c.bench_function("vec_set_create 100", |b| b.iter(|| vec_set_create(black_box(100))));
    c.bench_function("btree_set_create 100", |b| b.iter(|| btree_set_create(black_box(100))));
    c.bench_function("hash_set_create 100", |b| b.iter(|| hash_set_create(black_box(100))));
}

criterion_group!(benches, creation);
criterion_main!(benches);
