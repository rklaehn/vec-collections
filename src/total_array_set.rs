use crate::ArraySet;
use std::fmt::Debug;
use std::ops::{BitAnd, BitOr, BitXor, Neg, Sub};

#[derive(Clone, Debug)]
pub struct TotalArraySet<T> {
    elements: ArraySet<T>,
    negated: bool,
}

impl<T> TotalArraySet<T> {
    fn new(elements: ArraySet<T>, negated: bool) -> Self {
        Self { elements, negated }
    }

    pub fn is_empty(&self) -> bool {
        !self.negated && self.elements.is_empty()
    }

    fn negate(self) -> Self {
        Self::new(self.elements, !self.negated)
    }

    pub fn empty() -> Self {
        Self::new(ArraySet::empty(), false)
    }

    pub fn all() -> Self {
        Self::new(ArraySet::empty(), true)
    }
}

impl<T> From<ArraySet<T>> for TotalArraySet<T> {
    fn from(value: ArraySet<T>) -> Self {
        Self::new(value, false)
    }
}

impl<T: Ord + Default + Copy + Debug> TotalArraySet<T> {
    pub fn shrink_to_fit(&mut self) {
        self.elements.shrink_to_fit()
    }

    pub fn contains(&self, value: &T) -> bool {
        self.negated ^ self.elements.contains(value)
    }

    pub fn is_superset(&self, that: &Self) -> bool {
        !self.is_subset(that)
    }

    pub fn is_subset(&self, that: &Self) -> bool {
        match (self.negated, that.negated) {
            (false, false) => self.elements.is_subset(&that.elements),
            (false, true) => self.elements.is_disjoint(&that.elements),
            (true, false) => false,
            (true, true) => self.elements.is_superset(&that.elements),
        }
    }

    pub fn is_disjoint(&self, that: &Self) -> bool {
        match (self.negated, that.negated) {
            (false, false) => self.elements.is_disjoint(&that.elements),
            (false, true) => self.elements.is_subset(&that.elements),
            (true, false) => self.elements.is_superset(&that.elements),
            (true, true) => false,
        }
    }

    pub fn xor(self, that: Self) -> Self {
        Self::new(self.elements ^ that.elements, self.negated ^ that.negated)
    }

    pub fn union(self, that: Self) -> Self {
        match (self.negated, that.negated) {
            // union of elements
            (false, false) => Self::new(self.elements | that.elements, false),
            // remove holes from that
            (false, true) => Self::new(that.elements - self.elements, true),
            // remove holes from self
            (true, false) => Self::new(self.elements - that.elements, true),
            // intersection of holes
            (true, true) => Self::new(that.elements & self.elements, true),
        }
    }

    pub fn intersection(self, that: Self) -> Self {
        match (self.negated, that.negated) {
            // intersection of elements
            (false, false) => Self::new(self.elements & that.elements, false),
            // remove elements from self
            (false, true) => Self::new(self.elements - that.elements, false),
            // remove elements from that
            (true, false) => Self::new(that.elements - self.elements, false),
            // union of elements
            (true, true) => Self::new(that.elements | self.elements, true),
        }
    }

    pub fn difference(self, that: Self) -> Self {
        match (self.negated, that.negated) {
            // intersection of elements
            (false, false) => Self::new(self.elements - that.elements, false),
            // keep only holes of that
            (false, true) => Self::new(self.elements & that.elements, false),
            // add holes from that
            (true, false) => Self::new(self.elements | that.elements, true),
            // union of elements
            (true, true) => Self::new(that.elements - self.elements, false),
        }
    }
}

impl<T: Ord + Default + Copy + Debug> BitAnd for TotalArraySet<T> {
    type Output = TotalArraySet<T>;
    fn bitand(self, rhs: Self) -> Self {
        self.intersection(rhs)
    }
}

impl<T: Ord + Default + Copy + Debug> BitOr for TotalArraySet<T> {
    type Output = TotalArraySet<T>;
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl<T: Ord + Default + Copy + Debug> BitXor for TotalArraySet<T> {
    type Output = TotalArraySet<T>;
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.xor(rhs)
    }
}

impl<T: Ord + Default + Copy + Debug> Sub for TotalArraySet<T> {
    type Output = TotalArraySet<T>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl<T: Ord + Default + Copy + Debug> Neg for TotalArraySet<T> {
    type Output = TotalArraySet<T>;
    fn neg(self) -> Self::Output {
        self.negate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::*;
    use std::collections::BTreeSet;

    type Test = TotalArraySet<i64>;

    impl<T: Arbitrary + Ord + Copy + Default + Debug> Arbitrary for TotalArraySet<T> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            TotalArraySet::new(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g))
        }
    }

    fn binary_op(a: &Test, b: &Test, r: &Test, op: impl Fn(bool, bool) -> bool) -> bool {
        let mut samples: BTreeSet<i64> = BTreeSet::new();
        samples.extend(a.elements.as_slice().iter().cloned());
        samples.extend(b.elements.as_slice().iter().cloned());
        samples.insert(std::i64::MIN);
        samples.iter().all(|e| {
            let expected = op(a.contains(e), b.contains(e));
            let actual = r.contains(e);
            if expected != actual {
                println!(
                    "{:?}!={:?} at {:?} {:?} {:?} {:?}",
                    expected, actual, e, a, b, r
                );
            }
            expected == actual
        })
    }

    fn binary_property(a: &Test, b: &Test, r: bool, op: impl Fn(bool, bool) -> bool) -> bool {
        let mut samples: BTreeSet<i64> = BTreeSet::new();
        samples.extend(a.elements.as_slice().iter().cloned());
        samples.extend(b.elements.as_slice().iter().cloned());
        samples.insert(std::i64::MIN);
        if r {
            samples.iter().all(|e| {
                let expected = op(a.contains(e), b.contains(e));
                if !expected {
                    println!(
                        "{:?} is false at {:?}\na {:?}\nb {:?}\nr {:?}",
                        expected, e, a, b, r
                    );
                }
                expected
            })
        } else {
            samples.iter().any(|e| !op(a.contains(e), b.contains(e)))
        }
    }

    quickcheck! {

        fn is_disjoint_sample(a: Test, b: Test) -> bool {
            binary_property(&a, &b, a.is_disjoint(&b), |a, b| !(a & b))
        }

        fn is_subset_sample(a: Test, b: Test) -> bool {
            binary_property(&a, &b, a.is_subset(&b), |a, b| !a | b)
        }

        fn union_sample(a: Test, b: Test) -> bool {
            binary_op(&a, &b, &(a.clone() | b.clone()), |a, b| a | b)
        }

        fn intersection_sample(a: Test, b: Test) -> bool {
            binary_op(&a, &b, &(a.clone() & b.clone()), |a, b| a & b)
        }

        fn xor_sample(a: Test, b: Test) -> bool {
            binary_op(&a, &b, &(a.clone() ^ b.clone()), |a, b| a ^ b)
        }

        fn diff_sample(a: Test, b: Test) -> bool {
            binary_op(&a, &b, &(a.clone() - b.clone()), |a, b| a & !b)
        }
    }
}
