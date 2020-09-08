use crate::VecSet;
use core::{
    fmt,
    fmt::{Debug, Write},
    hash::Hash,
    mem,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign},
};
use smallvec::Array;

/// A [VecSet] with an additional flag so it can support negation.
///
/// This way it is possible to represent e.g. the set of all u64 except 1.
///
/// [VecSet]: struct.VecSet.html
pub struct TotalVecSet<A: Array> {
    elements: VecSet<A>,
    negated: bool,
}

/// Type alias for a [TotalVecSet](struct.TotalVecSet) with up to 2 elements with inline storage.
pub type TotalVecSet2<T> = TotalVecSet<[T; 2]>;

impl<T: Clone, A: Array<Item = T>> Clone for TotalVecSet<A> {
    fn clone(&self) -> Self {
        Self {
            elements: self.elements.clone(),
            negated: self.negated,
        }
    }
}

impl<T: Hash, A: Array<Item = T>> Hash for TotalVecSet<A> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.elements.hash(state);
        self.negated.hash(state);
    }
}

impl<T: PartialEq, A: Array<Item = T>> PartialEq for TotalVecSet<A> {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements && self.negated == other.negated
    }
}

impl<T: Eq, A: Array<Item = T>> Eq for TotalVecSet<A> {}

impl<T: Debug, A: Array<Item = T>> Debug for TotalVecSet<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.negated {
            f.write_char('!')?;
        }
        f.debug_set().entries(self.elements.iter()).finish()
    }
}

impl<T, A: Array<Item = T>> TotalVecSet<A> {
    fn new(elements: VecSet<A>, negated: bool) -> Self {
        Self { elements, negated }
    }

    pub fn is_empty(&self) -> bool {
        !self.negated && self.elements.is_empty()
    }

    pub fn is_all(&self) -> bool {
        self.negated && self.elements.is_empty()
    }

    pub fn constant(value: bool) -> Self {
        Self::new(VecSet::empty(), value)
    }

    pub fn empty() -> Self {
        false.into()
    }

    pub fn all() -> Self {
        true.into()
    }

    pub fn shrink_to_fit(&mut self) {
        self.elements.shrink_to_fit()
    }
}

impl<T, A: Array<Item = T>> From<bool> for TotalVecSet<A> {
    fn from(value: bool) -> Self {
        Self::constant(value)
    }
}

impl<T, A: Array<Item = T>> From<VecSet<A>> for TotalVecSet<A> {
    fn from(value: VecSet<A>) -> Self {
        Self::new(value, false)
    }
}

impl<T: Ord, A: Array<Item = T>> TotalVecSet<A> {
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

impl<T: Ord + Clone, A: Array<Item = T>> TotalVecSet<A> {
    pub fn remove(&mut self, that: &T) {
        if self.negated {
            self.elements.insert(that.clone())
        } else {
            self.elements.remove(that)
        }
    }
}

impl<T: Ord + Clone, A: Array<Item = T>> BitAnd for &TotalVecSet<A> {
    type Output = TotalVecSet<A>;
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

impl<T: Ord, A: Array<Item = T>> BitAndAssign for TotalVecSet<A> {
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
                mem::swap(&mut that.elements, &mut self.elements);
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

impl<T: Ord + Clone, A: Array<Item = T>> BitOr for &TotalVecSet<A> {
    type Output = TotalVecSet<A>;
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

impl<T: Ord, A: Array<Item = T>> BitOrAssign for TotalVecSet<A> {
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
                mem::swap(&mut that.elements, &mut self.elements);
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

impl<T: Ord + Clone, A: Array<Item = T>> BitXor for &TotalVecSet<A> {
    type Output = TotalVecSet<A>;
    fn bitxor(self, that: Self) -> Self::Output {
        Self::Output::new(&self.elements ^ &that.elements, self.negated ^ that.negated)
    }
}

impl<T: Ord, A: Array<Item = T>> BitXorAssign for TotalVecSet<A> {
    fn bitxor_assign(&mut self, that: Self) {
        self.elements ^= that.elements;
        self.negated ^= that.negated;
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl<T: Ord + Clone, A: Array<Item = T>> Sub for &TotalVecSet<A> {
    type Output = TotalVecSet<A>;
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

impl<T: Ord, A: Array<Item = T>> SubAssign for TotalVecSet<A> {
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
                mem::swap(&mut that.elements, &mut self.elements);
                self.elements -= that.elements;
                self.negated = false;
            }
        }
    }
}

impl<T: Ord + Clone, A: Array<Item = T>> Not for &TotalVecSet<A> {
    type Output = TotalVecSet<A>;
    fn not(self) -> Self::Output {
        Self::Output::new(self.elements.clone(), !self.negated)
    }
}

impl<T: Ord, A: Array<Item = T>> Not for TotalVecSet<A> {
    type Output = TotalVecSet<A>;
    fn not(self) -> Self::Output {
        Self::Output::new(self.elements, !self.negated)
    }
}

#[cfg(test)]
mod tests {
    #[allow(dead_code)]
    use super::*;
    use quickcheck::*;
    use std::collections::BTreeSet;

    type Test = TotalVecSet<[i64; 2]>;

    impl<T: Arbitrary + Ord + Copy + Default + Debug> Arbitrary for TotalVecSet<[T; 2]> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut elements: Vec<T> = Arbitrary::arbitrary(g);
            elements.truncate(2);
            let negated: bool = Arbitrary::arbitrary(g);
            TotalVecSet::new(elements.into(), negated)
        }
    }

    #[allow(dead_code)]
    /// just a helper to get good output when a check fails
    fn print_on_failure_unary<E: Debug, R: Eq + Debug>(x: E, expected: R, actual: R) -> bool {
        let res = expected == actual;
        if !res {
            println!("x:{:?} expected:{:?} actual:{:?}", x, expected, actual);
        }
        res
    }

    fn binary_op(a: &Test, b: &Test, r: &Test, op: impl Fn(bool, bool) -> bool) -> bool {
        let mut samples: BTreeSet<i64> = BTreeSet::new();
        samples.extend(a.elements.as_ref().iter().cloned());
        samples.extend(b.elements.as_ref().iter().cloned());
        samples.insert(core::i64::MIN);
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
        samples.extend(a.elements.as_ref().iter().cloned());
        samples.extend(b.elements.as_ref().iter().cloned());
        samples.insert(core::i64::MIN);
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
    bitop_empty!(Test);
    bitop_sub_not_all!(Test);
}
