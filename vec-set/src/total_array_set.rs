use crate::ArraySet;
use std::fmt::Debug;
use std::fmt::Write;
use std::ops::BitAndAssign;
use std::ops::BitOrAssign;
use std::ops::BitXorAssign;
use std::ops::SubAssign;
use std::ops::{BitAnd, BitOr, BitXor, Not, Sub};

#[derive(Clone, PartialEq, Eq)]
pub struct TotalArraySet<T> {
    elements: ArraySet<T>,
    negated: bool,
}

impl<T: Debug> Debug for TotalArraySet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.negated {
            f.write_char('!')?;
        }
        f.debug_set().entries(self.elements.iter()).finish()
    }
}

impl<T> TotalArraySet<T> {
    fn new(elements: ArraySet<T>, negated: bool) -> Self {
        Self { elements, negated }
    }

    pub fn is_empty(&self) -> bool {
        !self.negated && self.elements.is_empty()
    }

    pub fn is_all(&self) -> bool {
        self.negated && self.elements.is_empty()
    }

    pub fn empty() -> Self {
        Self::new(ArraySet::empty(), false)
    }

    pub fn all() -> Self {
        Self::new(ArraySet::empty(), true)
    }

    pub fn shrink_to_fit(&mut self) {
        self.elements.shrink_to_fit()
    }
}

impl<T> From<ArraySet<T>> for TotalArraySet<T> {
    fn from(value: ArraySet<T>) -> Self {
        Self::new(value, false)
    }
}

impl<T: Ord> TotalArraySet<T> {
    pub fn contains(&self, value: &T) -> bool {
        self.negated ^ self.elements.contains(value)
    }

    pub fn insert(&mut self, that: T) {
        if !self.negated {
            self.elements.insert(that)
        } else {
            self.elements.remove(&that)
        }
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
}

impl<T: Ord + Clone> TotalArraySet<T> {
    pub fn remove(&mut self, that: &T) {
        if self.negated {
            self.elements.insert(that.clone())
        } else {
            self.elements.remove(that)
        }
    }
}

impl<T: Ord + Clone> BitAnd for &TotalArraySet<T> {
    type Output = TotalArraySet<T>;
    fn bitand(self, that: Self) -> Self::Output {
        match (self.negated, that.negated) {
            // intersection of elements
            (false, false) => Self::Output::new(&self.elements & &that.elements, false),
            // remove elements from self
            (false, true) => Self::Output::new(&self.elements - &that.elements, false),
            // remove elements from that
            (true, false) => Self::Output::new(&that.elements - &self.elements, false),
            // union of elements
            (true, true) => Self::Output::new(&that.elements | &self.elements, true),
        }
    }
}

impl<T: Ord> BitAndAssign for TotalArraySet<T> {
    fn bitand_assign(&mut self, that: Self) {
        match (self.negated, that.negated) {
            // intersection of elements
            (false, false) => {
                self.elements &= that.elements;
                self.negated = false;
            }
            // remove elements from self
            (false, true) => {
                self.elements -= that.elements;
                self.negated = false;
            }
            // remove elements from that
            (true, false) => {
                let mut that = that;
                std::mem::swap(&mut that.elements, &mut self.elements);
                self.elements -= that.elements;
                self.negated = false;
            }
            // union of elements
            (true, true) => {
                self.elements |= that.elements;
                self.negated = true;
            }
        };
    }
}

impl<T: Ord + Clone> BitOr for &TotalArraySet<T> {
    type Output = TotalArraySet<T>;
    fn bitor(self, that: Self) -> Self::Output {
        match (self.negated, that.negated) {
            // union of elements
            (false, false) => Self::Output::new(&self.elements | &that.elements, false),
            // remove holes from that
            (false, true) => Self::Output::new(&that.elements - &self.elements, true),
            // remove holes from self
            (true, false) => Self::Output::new(&self.elements - &that.elements, true),
            // intersection of holes
            (true, true) => Self::Output::new(&that.elements & &self.elements, true),
        }
    }
}

impl<T: Ord> BitOrAssign for TotalArraySet<T> {
    fn bitor_assign(&mut self, that: Self) {
        match (self.negated, that.negated) {
            // union of elements
            (false, false) => {
                self.elements |= that.elements;
                self.negated = false;
            }
            // remove holes from that
            (false, true) => {
                let mut that = that;
                std::mem::swap(&mut that.elements, &mut self.elements);
                self.elements -= that.elements;
                self.negated = true;
            }
            // remove holes from self
            (true, false) => {
                self.elements -= that.elements;
                self.negated = true;
            }
            // intersection of holes
            (true, true) => {
                self.elements &= that.elements;
                self.negated = true;
            }
        };
    }
}

impl<T: Ord + Clone> BitXor for &TotalArraySet<T> {
    type Output = TotalArraySet<T>;
    fn bitxor(self, that: Self) -> Self::Output {
        Self::Output::new(&self.elements ^ &that.elements, self.negated ^ that.negated)
    }
}

impl<T: Ord> BitXorAssign for TotalArraySet<T> {
    fn bitxor_assign(&mut self, that: Self) {
        self.elements ^= that.elements;
        self.negated ^= that.negated;
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<T: Ord + Clone> Sub for &TotalArraySet<T> {
    type Output = TotalArraySet<T>;
    fn sub(self, that: Self) -> Self::Output {
        match (self.negated, that.negated) {
            // intersection of elements
            (false, false) => Self::Output::new(&self.elements - &that.elements, false),
            // keep only holes of that
            (false, true) => Self::Output::new(&self.elements & &that.elements, false),
            // add holes from that
            (true, false) => Self::Output::new(&self.elements | &that.elements, true),
            // union of elements
            (true, true) => Self::Output::new(&that.elements - &self.elements, false),
        }
    }
}

impl<T: Ord> SubAssign for TotalArraySet<T> {
    fn sub_assign(&mut self, that: Self) {
        match (self.negated, that.negated) {
            // intersection of elements
            (false, false) => {
                self.elements -= that.elements;
                self.negated = false;
            }
            // keep only holes of that
            (false, true) => {
                self.elements &= that.elements;
                self.negated = false;
            }
            // add holes from that
            (true, false) => {
                self.elements |= that.elements;
                self.negated = true;
            }
            // union of elements
            (true, true) => {
                let mut that = that;
                std::mem::swap(&mut that.elements, &mut self.elements);
                self.elements -= that.elements;
                self.negated = false;
            }
        }
    }
}

impl<T: Ord + Clone> Not for &TotalArraySet<T> {
    type Output = TotalArraySet<T>;
    fn not(self) -> Self::Output {
        Self::Output::new(self.elements.clone(), !self.negated)
    }
}

impl<T: Ord> Not for TotalArraySet<T> {
    type Output = TotalArraySet<T>;
    fn not(self) -> Self::Output {
        Self::Output::new(self.elements, !self.negated)
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
            let mut elements: Vec<T> = Arbitrary::arbitrary(g);
            elements.truncate(2);
            let negated: bool = Arbitrary::arbitrary(g);
            TotalArraySet::new(elements.into(), negated)
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
            binary_op(&a, &b, &(&a | &b), |a, b| a | b)
        }

        fn intersection_sample(a: Test, b: Test) -> bool {
            binary_op(&a, &b, &(&a & &b), |a, b| a & b)
        }

        fn xor_sample(a: Test, b: Test) -> bool {
            binary_op(&a, &b, &(&a ^ &b), |a, b| a ^ b)
        }

        fn diff_sample(a: Test, b: Test) -> bool {
            binary_op(&a, &b, &(&a - &b), |a, b| a & !b)
        }
    }

    bitop_assign_consistent!(Test);
    bitop_symmetry!(Test);
    empty_neutral!(Test);
    all_neutral!(Test);
}
