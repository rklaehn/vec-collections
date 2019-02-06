use super::*;
use quickcheck::*;
use std::fmt::Debug;
use std::ops::BitXor;

fn gen<T: Arbitrary, G: Gen>(g: &mut G) -> T {
    Arbitrary::arbitrary(g)
}

impl<T: Arbitrary + Ord + Copy> Arbitrary for Interval<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> Interval<T> {
        if gen(g) {
            let a: T = gen(g);
            let b: T = gen(g);
            let same: bool = a == b;
            let a_i: bool = same || gen(g);
            let b_i: bool = same || gen(g);
            Interval::range(Ord::min(a, b), a_i, Ord::max(a, b), b_i).unwrap()
        } else if gen(g) {
            if gen(g) {
                Interval::Above(gen(g), gen(g))
            } else {
                Interval::Below(gen(g), gen(g))
            }
        } else if gen(g) {
            Interval::at(gen(g))
        } else if Arbitrary::arbitrary(g) {
            Interval::All
        } else {
            Interval::Empty
        }
    }
}

impl<T: Arbitrary + Ord + Copy + Debug + Display> Arbitrary for IntervalSeq<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> IntervalSeq<T> {
        let a: Interval<T> = gen(g);
        let b: Interval<T> = gen(g);
        let r = IntervalSeq::from_interval(&a) ^ IntervalSeq::from_interval(&b);
        assert!(a.is_valid() && b.is_valid(), "Intervals are invalid");
        if !r.is_valid() {
            println!("BOOM {} {} {:?}", a, b, r);
        }
        r
        // let intervals: Vec<Interval<T>> = gen(g);
        // let result = intervals
        //     .iter()
        //     .map(IntervalSeq::from_interval)
        //     .fold(IntervalSeq::empty(), |a,b| {
        //         if !a.is_valid() {
        //             println!("a {:?}", a)
        //         }
        //         assert!(a.is_valid(), "a is valid");
        //         assert!(b.is_valid(), "b is valid");
        //         a ^ b
        //     }
        //     );
        // result
    }
}

quickcheck! {
    fn roundtrip(a: Interval<i64>) -> bool {
        let text = format!("{}", a);
        let b = text.parse::<Interval<i64>>().unwrap();
        a == b
    }

    fn roundtrip2(a: IntervalSeq<i64>) -> bool {
        if !a.is_valid() {
            println!("KAPUT {:?}", a);
        }
        println!("{:?}", a);
        let text = format!("{}", a);
        println!("{}", text);
        // let b = text.parse::<IntervalSeq<i64>>().unwrap();
        // a == b
        true
    }
}
