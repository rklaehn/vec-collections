extern crate vec_collections;

use std::collections::{BTreeSet, HashSet};
use vec_collections::VecSet;

// #[macro_use]
// extern crate lazy_static;
// extern crate rand;
// use rand::Rng;
// type Element = Vec<u8>;

// lazy_static! {
//     static ref ELEMENTS: Vec<Element> = make_elements();
// }

// fn make_elements() -> Vec<Element> {
//     let mut rng = rand::thread_rng();
//     (0..100000).map(move |_| {
//         // add zeros at the start
//         let mut random_bytes: Vec<u8> = (0..990).map(|_| 0).collect();
//         random_bytes.extend((0..10).map(|_| { rng.gen::<u8>() }));
//         random_bytes
//     }).collect()
// }

// fn element(x: usize) -> Element {
//     ELEMENTS.get(x).unwrap().clone()
// }

type Element = u32;

fn element(x: usize) -> Element {
    x as Element
}


struct TestData {
    params: String,
    a: Vec<Element>,
    b: Vec<Element>,
}

impl TestData {
    fn interleaved(n: usize) -> TestData {
        TestData {
            params: format!("interleaved {}", n),
            a: (0..n).map(|x| element(2 * x)).collect(),
            b: (0..n).map(|x| element(2 * x + 1)).collect(),
        }
    }

    fn non_overlapping(n: usize) -> TestData {
        TestData {
            params: format!("non_overlapping {}", n),
            a: (0..n).map(|x| element(2 * x)).collect(),
            b: (0..n).map(|x| element(2 * x + 2 * n)).collect(),
        }
    }
}

fn union_arrayset(data: &TestData) {
    let a: VecSet<Element> = data.a.clone().into();
    let b: VecSet<Element> = data.b.clone().into();
    let t0 = std::time::Instant::now();
    let _r = &a | &b;
    let dt = std::time::Instant::now() - t0;
    println!("union vecset {} {:?}", data.params, dt);
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
    let a: VecSet<Element> = data.a.clone().into();
    let b: VecSet<Element> = data.b.clone().into();
    let t0 = std::time::Instant::now();
    let _r = &a & &b;
    let dt = std::time::Instant::now() - t0;
    println!("intersection vecset {} {:?}", data.params, dt);
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
    let a: VecSet<Element> = data.a.clone().into();
    let b: VecSet<Element> = data.b.clone().into();
    let t0 = std::time::Instant::now();
    let _r = a.is_disjoint(&b);
    let dt = std::time::Instant::now() - t0;
    println!("is_disjoint vecset {} {} {:?}", _r, data.params, dt);
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
fn creation_arrayset(name: &str, data: &Vec<Element>) {
    let elems = data.clone();
    let t0 = std::time::Instant::now();
    let a: VecSet<Element> = elems.into_iter().collect();
    let dt = std::time::Instant::now() - t0;
    println!("creation vecset {} {} {:?}", a.len(), name, dt);
}
fn creation_btreeset(name: &str, data: &Vec<Element>) {
    let elems = data.clone();
    let t0 = std::time::Instant::now();
    let a: BTreeSet<Element> = elems.into_iter().collect();
    let dt = std::time::Instant::now() - t0;
    println!("creation btreeset {} {} {:?}", a.len(), name, dt);
}
fn creation_hashset(name: &str, data: &Vec<Element>) {
    let elems = data.clone();
    let t0 = std::time::Instant::now();
    let a: HashSet<Element> = elems.into_iter().collect();
    let dt = std::time::Instant::now() - t0;
    println!("creation hashset {} {} {:?}", a.len(), name, dt);
}
fn main() {
    let interleaved = TestData::interleaved(10000);

    let non_overlapping = TestData::non_overlapping(10000);

    let mut x: Vec<Element> = Vec::new();
    for i in 0..1000000 {
        x.push(element(i * 3 % 100000));
    }

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

    creation_arrayset("mixed", &x);
    creation_btreeset("mixed", &x);
    creation_hashset("mixed", &x);
}
