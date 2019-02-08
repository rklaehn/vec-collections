use super::*;
use std::time::Instant;

fn make_profile_xor(below_all: bool, support: &Vec<i32>) -> IntervalSeq<i32> {
    let mut result: IntervalSeq<i32> = IntervalSeq::from(below_all);
    for x in support {
        result = result ^ IntervalSeq::at_or_above(*x);
    }
    result
}

fn make_on_off_profile(n: i32, offset: i32, stride: i32) -> IntervalSeq<i32> {
    let support = (0..n).map(|x| x * stride + offset).collect::<Vec<i32>>();
    make_profile_xor(false, &support)
}

fn bench<F, U>(name: &str, f: F) -> Vec<U>
where
    F: Fn() -> U,
{
    let mut res: Vec<U> = Vec::new();
    let n = 1000;
    let t0 = Instant::now();
    for _ in 0..n {
        res.push(f());
    }
    let dt = Instant::now() - t0;
    println!("{}\t{} ns", name, dt.as_nanos() / n);
    res
}

fn full_traversal_bench(n: i32) -> () {
    println!("Full traversal benchmark (n={})", n);
    let a = make_on_off_profile(n, 0, 2);
    let b = make_on_off_profile(n, 1, 2);
    bench("or", || a.clone() | b.clone());
    bench("and", || a.clone() & b.clone());
    bench("xor", || a.clone() & b.clone());
}

fn cutoff_bench(n: i32) -> () {
    println!("Cutoff benchmark (n={})", n);
    let a = make_on_off_profile(n, 0, 2);
    let b = make_on_off_profile(n, 1, 1000);
    bench("or", || a.clone() | b.clone());
    bench("and", || a.clone() & b.clone());
    bench("xor", || a.clone() & b.clone());
}

#[test]
fn full_traversal_bench_large() {
    full_traversal_bench(100000);
}

#[test]
fn cutoff_bench_large() {
    cutoff_bench(100000);
}
