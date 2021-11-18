use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use vec_collections::{vecset, AbstractVecSet, VecSet};

type TestSet = VecSet<[u64; 4]>;

fn union(a: &TestSet, b: &TestSet) -> TestSet {
    a | b
}

fn union_small(c: &mut Criterion) {
    let a = vecset! {1,2,3};
    let b = vecset! {2,3,4};
    c.bench_with_input(
        BenchmarkId::new("union", 0),
        &(&a, &b),
        |bencher, (a, b)| bencher.iter(|| union(black_box(a), black_box(b))),
    );
}

fn is_subset_small(c: &mut Criterion) {
    let a: TestSet = vecset! {1,2,3,4};
    let b: TestSet = vecset! {2,3,4};
    c.bench_with_input(
        BenchmarkId::new("is_subset", 0),
        &(&a, &b),
        |bencher, (a, b)| bencher.iter(|| black_box(a).is_subset(black_box(*b))),
    );
}

criterion_group!(benches, union_small, is_subset_small);
criterion_main!(benches);
