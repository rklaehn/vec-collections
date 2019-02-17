#[macro_use]
extern crate bencher;

use bencher::Bencher;
use intervalset::IntervalSeq;

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

fn full_traversal_and(bench: &mut Bencher) {
    let n: i32 = 100;
    let a = make_on_off_profile(n, 0, 2);
    let b = make_on_off_profile(n, 1, 2);
    bench.iter(|| a.clone() | b.clone())
}

benchmark_group!(benches, full_traversal_and);
benchmark_main!(benches);
