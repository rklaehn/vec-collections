extern crate abc;

use abc::ArraySet;
use std::collections::{BTreeSet, HashSet};

struct TestData {
    a: Vec<i32>,
    b: Vec<i32>,
}

impl TestData {
    fn interleaved(n: i32) -> TestData {
        TestData {
            a: (0..n).map(|x| 2 * x).collect(),
            b: (0..n).map(|x| 2 * x + 1).collect(),
        }
    }

    fn non_overlapping(n: i32) -> TestData {
        TestData {
            a: (0..n).map(|x| 2 * x).collect(),
            b: (0..n).map(|x| 2 * x + n).collect(),
        }
    }
}

fn union_arrayset(data: &TestData) {
    let a: ArraySet<i32> = data.a.iter().cloned().collect();
    let b: ArraySet<i32> = data.b.iter().cloned().collect();
    let t0 = std::time::Instant::now();
    let _r = a.clone() | b.clone();
    let dt = std::time::Instant::now() - t0;
    println!("{:?}", dt);
}

fn union_btreeset(data: &TestData) {
    let a: BTreeSet<i32> = data.a.iter().cloned().collect();
    let b: BTreeSet<i32> = data.b.iter().cloned().collect();
    let t0 = std::time::Instant::now();
    let _r = &a | &b;
    let dt = std::time::Instant::now() - t0;
    println!("{:?}", dt);
}

fn union_hashset(data: &TestData) {
    let a: HashSet<i32> = data.a.iter().cloned().collect();
    let b: HashSet<i32> = data.b.iter().cloned().collect();
    let t0 = std::time::Instant::now();
    let _r = &a | &b;
    let dt = std::time::Instant::now() - t0;
    println!("{:?}", dt);
}
fn main() {
    let interleaved = TestData::interleaved(10000);
    union_arrayset(&interleaved);
    union_btreeset(&interleaved);
    union_hashset(&interleaved);

    let non_overlapping = TestData::non_overlapping(10000);
    union_arrayset(&non_overlapping);
    union_btreeset(&non_overlapping);
    union_hashset(&non_overlapping);
}