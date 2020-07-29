extern crate vec_collections;

use std::collections::{BTreeMap, HashMap};
use vec_collections::VecMap2;

type Element = i64;
fn element(x: usize) -> Element {
    x as Element
}

fn creation_vecmap(name: &str, data: &Vec<(Element, Element)>) {
    let elems = data.clone();
    let t0 = std::time::Instant::now();
    let a: VecMap2<Element, Element> = elems.into_iter().collect();
    let dt = std::time::Instant::now() - t0;
    println!("creation vecmap {} {} {:?}", a.len(), name, dt);
}
fn creation_btreemap(name: &str, data: &Vec<(Element, Element)>) {
    let elems = data.clone();
    let t0 = std::time::Instant::now();
    let a: BTreeMap<Element, Element> = elems.into_iter().collect();
    let dt = std::time::Instant::now() - t0;
    println!("creation btreemap {} {} {:?}", a.len(), name, dt);
}
fn creation_hashmap(name: &str, data: &Vec<(Element, Element)>) {
    let elems = data.clone();
    let t0 = std::time::Instant::now();
    let a: HashMap<Element, Element> = elems.into_iter().collect();
    let dt = std::time::Instant::now() - t0;
    println!("creation hashmap {} {} {:?}", a.len(), name, dt);
}
fn main() {
    let mut x: Vec<(Element, Element)> = Vec::new();
    for i in 0..1000000 {
        x.push((element(i * 3 % 100000), element(i)));
    }

    for _ in 0..10 {
        creation_vecmap("mixed", &x);
        creation_btreemap("mixed", &x);
        creation_hashmap("mixed", &x);
    }
}
