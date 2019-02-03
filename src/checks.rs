use super::*;
use quickcheck::*;

fn gen<T: Arbitrary, G: Gen>(g: &mut G) -> T {
    Arbitrary::arbitrary(g)
}

impl<T: Arbitrary + Eq + Ord + Copy> Arbitrary for Interval<T> {
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

quickcheck! {
    fn roundtrip(a: Interval<i64>) -> bool {
        let text = format!("{}", a);
        let b = text.parse::<Interval<i64>>().unwrap();
        a == b
    }
}
