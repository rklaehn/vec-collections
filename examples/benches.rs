extern crate abc;

use abc::ArraySet;
use std::collections::{BTreeSet, HashSet};

type Element = i32;

struct TestData {
    params: String,
    a: Vec<Element>,
    b: Vec<Element>,
}

impl TestData {
    fn interleaved(n: Element) -> TestData {
        TestData {
            params: format!("interleaved {}", n),
            a: (0..n).map(|x| 2 * x).collect(),
            b: (0..n).map(|x| 2 * x + 1).collect(),
        }
    }

    fn non_overlapping(n: Element) -> TestData {
        TestData {
            params: format!("non_overlapping {}", n),
            a: (0..n).map(|x| 2 * x).collect(),
            b: (0..n).map(|x| 2 * x + 2 * n).collect(),
        }
    }
}

fn union_arrayset(data: &TestData) {
    let a: ArraySet<Element> = data.a.clone().into();
    let b: ArraySet<Element> = data.b.clone().into();
    let t0 = std::time::Instant::now();
    let _r = &a | &b;
    let dt = std::time::Instant::now() - t0;
    println!("union arrayset {} {:?}", data.params, dt);
}

fn union_btreeset(data: &TestData) {
    let a: BTreeSet<Element> = data.a.iter().cloned().collect();
    let b: BTreeSet<Element> = data.b.iter().cloned().collect();
    let t0 = std::time::Instant::now();
    let _r = &a | &b;
    let dt = std::time::Instant::now() - t0;
    println!("union btreeset {} {:?}", data.params, dt);
}

fn union_hashset(data: &TestData) {
    let a: HashSet<Element> = data.a.iter().cloned().collect();
    let b: HashSet<Element> = data.b.iter().cloned().collect();
    let t0 = std::time::Instant::now();
    let _r = &a | &b;
    let dt = std::time::Instant::now() - t0;
    println!("union hashset {} {:?}", data.params, dt);
}

fn intersection_arrayset(data: &TestData) {
    let a: ArraySet<Element> = data.a.clone().into();
    let b: ArraySet<Element> = data.b.clone().into();
    let t0 = std::time::Instant::now();
    let _r = &a & &b;
    let dt = std::time::Instant::now() - t0;
    println!("intersection arrayset {} {:?}", data.params, dt);
}

fn intersection_btreeset(data: &TestData) {
    let a: BTreeSet<Element> = data.a.iter().cloned().collect();
    let b: BTreeSet<Element> = data.b.iter().cloned().collect();
    let t0 = std::time::Instant::now();
    let _r = &a & &b;
    let dt = std::time::Instant::now() - t0;
    println!("intersection btreeset {} {:?}", data.params, dt);
}

fn intersection_hashset(data: &TestData) {
    let a: HashSet<Element> = data.a.iter().cloned().collect();
    let b: HashSet<Element> = data.b.iter().cloned().collect();
    let t0 = std::time::Instant::now();
    let _r = &a & &b;
    let dt = std::time::Instant::now() - t0;
    println!("intersection hashset {} {:?}", data.params, dt);
}

fn is_disjoint_arrayset(data: &TestData) {
    let a: ArraySet<Element> = data.a.clone().into();
    let b: ArraySet<Element> = data.b.clone().into();
    let t0 = std::time::Instant::now();
    let _r = a.is_disjoint(&b);
    let dt = std::time::Instant::now() - t0;
    println!("is_disjoint arrayset {} {} {:?}", _r, data.params, dt);
}

fn is_disjoint_btreeset(data: &TestData) {
    let a: BTreeSet<Element> = data.a.iter().cloned().collect();
    let b: BTreeSet<Element> = data.b.iter().cloned().collect();
    let t0 = std::time::Instant::now();
    let _r = a.is_disjoint(&b);
    let dt = std::time::Instant::now() - t0;
    println!("is_disjoint btreeset {} {} {:?}", _r, data.params, dt);
}

fn is_disjoint_hashset(data: &TestData) {
    let a: HashSet<Element> = data.a.iter().cloned().collect();
    let b: HashSet<Element> = data.b.iter().cloned().collect();
    let t0 = std::time::Instant::now();
    let _r = a.is_disjoint(&b);
    let dt = std::time::Instant::now() - t0;
    println!("is_disjoint hashset {} {} {:?}", _r, data.params, dt);
}
fn main() {
    let interleaved = TestData::interleaved(10000);

    let non_overlapping = TestData::non_overlapping(10000);

    union_arrayset(&interleaved);
    union_btreeset(&interleaved);
    union_hashset(&interleaved);

    union_arrayset(&non_overlapping);
    union_btreeset(&non_overlapping);
    union_hashset(&non_overlapping);

    intersection_arrayset(&interleaved);
    intersection_btreeset(&interleaved);
    intersection_hashset(&interleaved);

    intersection_arrayset(&non_overlapping);
    intersection_btreeset(&non_overlapping);
    intersection_hashset(&non_overlapping);

    is_disjoint_arrayset(&interleaved);
    is_disjoint_btreeset(&interleaved);
    is_disjoint_hashset(&interleaved);

    is_disjoint_arrayset(&non_overlapping);
    is_disjoint_btreeset(&non_overlapping);
    is_disjoint_hashset(&non_overlapping);
}
