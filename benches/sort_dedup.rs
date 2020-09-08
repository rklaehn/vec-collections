// use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
// use std::collections::{BTreeSet, HashSet};
// use vec_collections::*;

// fn vs_create(v: &Vec<u64>) -> usize {
//     let res: Vec<u64> = sort_and_dedup(v.iter().cloned());
//     res.len()
// }

// fn bs_create(v: &Vec<u64>) -> usize {
//     let res: BTreeSet<u64> = v.iter().cloned().collect();
//     let res: Vec<_> = res.into_iter().collect();
//     res.len()
// }

// fn hs_create(v: &Vec<u64>) -> usize {
//     let res: HashSet<u64> = v.iter().cloned().collect();
//     let res: Vec<_> = res.into_iter().collect();
//     res.len()
// }

// use rand::seq::SliceRandom;
// use rand::SeedableRng;

// pub fn creation(c: &mut Criterion) {
//     let mut group = c.benchmark_group("sort_dedup");
//     let mut rand = rand::rngs::StdRng::from_seed([0u8; 32]);
//     for i in (100u64..=1000).step_by(100) {
//         let mut values = (0..i).collect::<Vec<_>>();
//         values.shuffle(&mut rand);

//         group.bench_with_input(BenchmarkId::new("via_vec", i), &values,
//             |b, values| b.iter(|| vs_create(black_box(values))));

//         group.bench_with_input(BenchmarkId::new("via_btree", i), &values,
//             |b, values| b.iter(|| bs_create(black_box(values))));

//         group.bench_with_input(BenchmarkId::new("via_hash", i), &values,
//             |b, values| b.iter(|| hs_create(black_box(values))));
//     }
// }

// criterion_group!(benches, creation);
// criterion_main!(benches);
