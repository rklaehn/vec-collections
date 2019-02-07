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
        let intervals: Vec<Interval<T>> = gen(g);
        intervals
            .iter()
            .map(IntervalSeq::from)
            .fold(IntervalSeq::empty(), BitXor::bitxor)
    }
}

fn sample_test<F>(a: &IntervalSeq<i64>, b: &IntervalSeq<i64>, r: &IntervalSeq<i64>, op: F) -> bool
where
    F: Fn(bool, bool) -> bool,
{
    let mut support: Vec<i64> = Vec::new();
    support.extend(a.clone().edges());
    support.extend(b.clone().edges());
    support.extend(r.clone().edges());
    support.dedup();
    support.iter().all(|x| {
        (op(a.below(*x), b.below(*x)) == r.below(*x)
            && op(a.at(*x), b.at(*x)) == r.at(*x)
            && op(a.above(*x), b.above(*x)) == r.above(*x))
    })
}

quickcheck! {
    fn interval_tostring_roundtrip(a: Interval<i64>) -> bool {
        let text = format!("{}", a);
        let b = text.parse::<Interval<i64>>().unwrap();
        a == b
    }

    fn intervalseq_tostring_roundtrip(a: IntervalSeq<i64>) -> bool {
        let text = format!("{}", a);
        let b = text.parse::<IntervalSeq<i64>>().unwrap();
        a == b
    }

    fn intervalseq_and_sample(a: IntervalSeq<i64>, b: IntervalSeq<i64>) -> bool {
        sample_test(&a.clone(), &b.clone(), &(a & b), |a, b| a & b)
    }

    fn intervalseq_or_sample(a: IntervalSeq<i64>, b: IntervalSeq<i64>) -> bool {
        sample_test(&a.clone(), &b.clone(), &(a | b), |a, b| a | b)
    }

    fn intervalseq_xor_sample(a: IntervalSeq<i64>, b: IntervalSeq<i64>) -> bool {
        sample_test(&a.clone(), &b.clone(), &(a ^ b), |a, b| a ^ b)
    }

    fn intervalseq_not_sample(a: IntervalSeq<i64>) -> bool {
        let r = !a.clone();
        let mut support: Vec<i64> = Vec::new();
        support.extend(a.clone().edges());
        support.extend(r.clone().edges());
        support.dedup();
        support.iter().all(|x| {
            !a.below(*x) == r.below(*x) &&
            !a.at(*x) == r.at(*x) &&
            !a.above(*x) == r.above(*x)
        })
    }
}
