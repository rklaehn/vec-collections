//! A set backed by a `SmallVec<T>`.
//!
//! An advantage of this set compared to e.g. BTreeSet is that small sets will be stored inline without allocations.
//! Larger sets will be stored using a single object on the heap.
//!
//! A disadvante is that insertion and removal of single elements is very slow (O(N)) for large sets.
//!
//! Set operations (union, intersection etc.) are supported using the binary operators, with both variants that
//! create new sets and in-place variants.
use crate::binary_merge::{EarlyOut, MergeOperation};
use crate::dedup::sort_and_dedup;
use crate::iterators::SortedIter;
use crate::merge_state::{BoolOpMergeState, InPlaceMergeState, MergeStateMut, SmallVecMergeState};
use smallvec::{Array, SmallVec};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::iter::FromIterator;
use std::{hash::Hash, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign}};

struct SetUnionOp;
struct SetIntersectionOp;
struct SetXorOp;
struct SetDiffOpt;

/// A set backed by a `SmallVec<T>`
#[derive(Default)]
pub struct VecSet<A: Array>(SmallVec<A>);

pub type VecSet2<T> = VecSet<[T; 2]>;

impl<T: Debug, A: Array<Item = T>> Debug for VecSet<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }   
}

impl<T: Clone, A: Array<Item = T>> Clone for VecSet<A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Hash, A: Array<Item = T>> Hash for VecSet<A> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: PartialEq, A: Array<Item = T>> PartialEq for VecSet<A> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq, A: Array<Item = T>> Eq for VecSet<A> {}

impl<T: PartialOrd, A: Array<Item = T>> PartialOrd for VecSet<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Ord, A: Array<Item = T>> Ord for VecSet<A> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<A: Array> VecSet<A> {
    /// private because it does not check the invariants
    fn new(a: SmallVec<A>) -> Self {
        Self(a)
    }
    /// a set with a single element. Will not allocate.
    pub fn singleton(value: A::Item) -> Self {
        let mut res = SmallVec::new();
        res.push(value);
        Self(res)
    }
    /// the empty set. Will not allocate.
    pub fn empty() -> Self {
        Self::new(SmallVec::new())
    }
    /// An iterator that returns references to the items of this set in sorted order
    pub fn iter(&self) -> SortedIter<std::slice::Iter<A::Item>> {
        SortedIter::new(self.0.iter())
    }
    /// The underlying memory as a slice
    pub fn as_slice(&self) -> &[A::Item] {
        &self.0
    }
    /// number of elements in the set
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// shrink the underlying SmallVec<T> to fit
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }
    /// true if the set is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<A: Array> VecSet<A>
    where A::Item: Ord {
    /// insert an element.
    ///
    /// The time complexity of this is O(N), so building a large set using inserts will be slow!
    pub fn insert(&mut self, that: A::Item) {
        match self.0.binary_search(&that) {
            Ok(index) => self.0[index] = that,
            Err(index) => self.0.insert(index, that),
        }
    }

    /// Remove an element
    ///
    /// The time complexity of this is O(N), so building a large set using inserts will be slow!
    pub fn remove(&mut self, that: &A::Item) {
        if let Ok(index) = self.0.binary_search(&that) {
            self.0.remove(index);
        };
    }

    /// true if this set has no common elements with another set
    pub fn is_disjoint(&self, that: &Self) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetIntersectionOp)
    }

    /// true if this set is a subset of another set
    ///
    /// A set is considered to be a subset of itself
    pub fn is_subset(&self, that: &Self) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetDiffOpt)
    }

    /// true if this set is a superset of another set
    ///
    /// A set is considered to be a superset of itself
    pub fn is_superset(&self, that: &Self) -> bool {
        that.is_subset(self)
    }

    /// true if this set contains the item.
    ///
    /// Time complexity is O(log N). Binary search.
    pub fn contains(&self, value: &A::Item) -> bool {
        self.0.binary_search(value).is_ok()
    }

    /// creates a set from a vec
    ///
    /// Will sort and deduplicate the vector using a stable merge sort, so worst case time complexity
    /// is O(N log N). However, this will be faster for an already partially sorted vector.
    ///
    /// Note that the backing memory of the vector will be reused, so if this is a large vector containing
    /// lots of duplicates, it is advisable to call shrink_to_fit on the resulting set.
    fn from_vec(vec: Vec<A::Item>) -> Self {
        let mut vec = vec;
        vec.sort();
        vec.dedup();
        Self::new(SmallVec::from_vec(vec))
    }
}

impl<A: Array> Into<Vec<A::Item>> for VecSet<A> {
    fn into(self) -> Vec<A::Item> {
        self.0.into_vec()
    }
}

impl<T: Ord + Clone, Arr: Array<Item = T>> BitAnd for &VecSet<Arr> {
    type Output = VecSet<Arr>;
    fn bitand(self, that: Self) -> Self::Output {
        Self::Output::new(SmallVecMergeState::merge(
            &self.0,
            &that.0,
            SetIntersectionOp,
        ))
    }
}

impl<T: Ord + Clone, Arr: Array<Item = T>> BitOr for &VecSet<Arr> {
    type Output = VecSet<Arr>;
    fn bitor(self, that: Self) -> Self::Output {
        Self::Output::new(SmallVecMergeState::merge(&self.0, &that.0, SetUnionOp))
    }
}

impl<T: Ord + Clone, Arr: Array<Item = T>> BitXor for &VecSet<Arr> {
    type Output = VecSet<Arr>;
    fn bitxor(self, that: Self) -> Self::Output {
        Self::Output::new(SmallVecMergeState::merge(&self.0, &that.0, SetXorOp))
    }
}

impl<T: Ord + Clone, Arr: Array<Item = T>> Sub for &VecSet<Arr> {
    type Output = VecSet<Arr>;
    fn sub(self, that: Self) -> Self::Output {
        Self::Output::new(SmallVecMergeState::merge(&self.0, &that.0, SetDiffOpt))
    }
}

impl<T: Ord, A: Array<Item = T>> BitAndAssign for VecSet<A> {
    fn bitand_assign(&mut self, that: Self) {
        InPlaceMergeState::merge(&mut self.0, that.0, SetIntersectionOp);
    }
}

impl<T: Ord, A: Array<Item = T>> BitOrAssign for VecSet<A> {
    fn bitor_assign(&mut self, that: Self) {
        InPlaceMergeState::merge(&mut self.0, that.0, SetUnionOp);
    }
}

impl<T: Ord, A: Array<Item = T>> BitXorAssign for VecSet<A> {
    fn bitxor_assign(&mut self, that: Self) {
        InPlaceMergeState::merge(&mut self.0, that.0, SetXorOp);
    }
}

impl<T: Ord, A: Array<Item = T>> SubAssign for VecSet<A> {
    fn sub_assign(&mut self, that: Self) {
        InPlaceMergeState::merge(&mut self.0, that.0, SetDiffOpt);
    }
}

impl<A: Array> AsRef<[A::Item]> for VecSet<A> {
    fn as_ref(&self) -> &[A::Item] {
        self.as_slice()
    }
}

impl<T: Ord, A: Array<Item=T>> From<Vec<T>> for VecSet<A> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}

impl<T: Ord, A: Array<Item=T>> From<BTreeSet<T>> for VecSet<A> {
    fn from(value: BTreeSet<T>) -> Self {
        Self::new(value.into_iter().collect())
    }
}

/// Builds the set from an iterator.
///
/// Uses a heuristic to deduplicate while building the set, so the intermediate storage will never be more
/// than twice the size of the resulting set.
impl<T: Ord, A: Array<Item=T>> FromIterator<T> for VecSet<A> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(sort_and_dedup(iter.into_iter()))
    }
}

impl<T: Ord, A: Array<Item=T>> Extend<T> for VecSet<A> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        *self |= Self::from_iter(iter);
    }
}

impl<T: Ord, I: MergeStateMut<A = T, B = T>> MergeOperation<I> for SetUnionOp {
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, true)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.advance_a(1, true)?;
        m.advance_b(1, false)
    }
}

impl<T: Ord, I: MergeStateMut<A = T, B = T>> MergeOperation<I> for SetIntersectionOp {
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, false)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, false)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.advance_a(1, true)?;
        m.advance_b(1, false)
    }
}

impl<T: Ord, I: MergeStateMut<A = T, B = T>> MergeOperation<I> for SetDiffOpt {
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, false)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.advance_a(1, false)?;
        m.advance_b(1, false)
    }
}

impl<T: Ord, I: MergeStateMut<A = T, B = T>> MergeOperation<I> for SetXorOp {
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, true)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.advance_a(1, false)?;
        m.advance_b(1, false)
    }
}

// impl<T: Ord> BitAnd for VecSet<T> {
//     type Output = VecSet<T>;
//     fn bitand(mut self, that: Self) -> Self::Output {
//         self &= that;
//         self
//     }
// }

// impl<T: Ord> BitOr for VecSet<T> {
//     type Output = VecSet<T>;
//     fn bitor(mut self, that: Self) -> Self::Output {
//         self |= that;
//         self
//     }
// }

// impl<T: Ord> BitXor for VecSet<T> {
//     type Output = VecSet<T>;
//     fn bitxor(mut self, that: Self) -> Self::Output {
//         self ^= that;
//         self
//     }
// }

// impl<T: Ord + Default + Copy> VecSet<T> {
//     pub fn union_with(&mut self, that: &VecSet<T>) {
//         InPlaceMergeState::merge(&mut self.0, &that.0, SetUnionOp());
//     }

//     pub fn intersection_with(&mut self, that: &VecSet<T>) {
//         InPlaceMergeState::merge(&mut self.0, &that.0, SetIntersectionOp());
//     }

//     pub fn xor_with(&mut self, that: &VecSet<T>) {
//         InPlaceMergeState::merge(&mut self.0, &that.0, SetIntersectionOp());
//     }

//     pub fn difference_with(&mut self, that: &VecSet<T>) {
//         InPlaceMergeState::merge(&mut self.0, &that.0, SetDiffOpt());
//     }
// }

#[cfg(test)]
mod test {
    use super::*;
    use crate::obey::*;
    use num_traits::PrimInt;
    use quickcheck::*;
    use std::collections::BTreeSet;

    impl<T: Arbitrary + Ord + Copy + Default + Debug> Arbitrary for VecSet<[T; 2]> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Self::from_vec(Arbitrary::arbitrary(g))
        }
    }

    impl<E: PrimInt> TestSamples<E, bool> for VecSet<[E; 2]> {
        fn samples(&self, res: &mut BTreeSet<E>) {
            res.insert(E::min_value());
            for x in self.0.iter().cloned() {
                res.insert(x - E::one());
                res.insert(x);
                res.insert(x + E::one());
            }
            res.insert(E::max_value());
        }

        fn at(&self, elem: E) -> bool {
            self.contains(&elem)
        }
    }

    type Test = VecSet<[i64; 2]>;
    type Reference = BTreeSet<i64>;

    quickcheck! {

        fn is_disjoint_sample(a: Test, b: Test) -> bool {
            binary_property_test(&a, &b, a.is_disjoint(&b), |a, b| !(a & b))
        }

        fn is_subset_sample(a: Test, b: Test) -> bool {
            binary_property_test(&a, &b, a.is_subset(&b), |a, b| !a | b)
        }

        fn union_sample(a: Test, b: Test) -> bool {
            binary_element_test(&a, &b, &a | &b, |a, b| a | b)
        }

        fn intersection_sample(a: Test, b: Test) -> bool {
            binary_element_test(&a, &b, &a & &b, |a, b| a & b)
        }

        fn xor_sample(a: Test, b: Test) -> bool {
            binary_element_test(&a, &b, &a ^ &b, |a, b| a ^ b)
        }

        fn diff_sample(a: Test, b: Test) -> bool {
            binary_element_test(&a, &b, &a - &b, |a, b| a & !b)
        }

        fn union(a: Reference, b: Reference) -> bool {
            let mut a1: Test = a.iter().cloned().collect();
            let b1: Test = b.iter().cloned().collect();
            let r2 = &a1 | &b1;
            a1 |= b1;
            println!("{:?} {:?}", a, b);
            let expected: Vec<i64> = a.union(&b).cloned().collect();
            let actual: Vec<i64> = a1.into();
            let actual2: Vec<i64> = r2.into();
            expected == actual && expected == actual2
        }

        fn intersection(a: Reference, b: Reference) -> bool {
            let mut a1: Test = a.iter().cloned().collect();
            let b1: Test = b.iter().cloned().collect();
            let r2 = &a1 & &b1;
            a1 &= b1;
            let expected: Vec<i64> = a.intersection(&b).cloned().collect();
            let actual: Vec<i64> = a1.into();
            let actual2: Vec<i64> = r2.into();
            expected == actual && expected == actual2
        }

        fn xor(a: Reference, b: Reference) -> bool {
            let mut a1: Test = a.iter().cloned().collect();
            let b1: Test = b.iter().cloned().collect();
            let r2 = &a1 ^ &b1;
            a1 ^= b1;
            let expected: Vec<i64> = a.symmetric_difference(&b).cloned().collect();
            let actual: Vec<i64> = a1.into();
            let actual2: Vec<i64> = r2.into();
            expected == actual && expected == actual2
        }

        fn difference(a: Reference, b: Reference) -> bool {
            let mut a1: Test = a.iter().cloned().collect();
            let b1: Test = b.iter().cloned().collect();
            let r2 = &a1 - &b1;
            a1 -= b1;
            let expected: Vec<i64> = a.difference(&b).cloned().collect();
            let actual: Vec<i64> = a1.into();
            let actual2: Vec<i64> = r2.into();
            expected == actual && expected == actual2
        }

        fn is_disjoint(a: Reference, b: Reference) -> bool {
            let a1: Test = a.iter().cloned().collect();
            let b1: Test = b.iter().cloned().collect();
            let actual = a1.is_disjoint(&b1);
            let expected = a.is_disjoint(&b);
            expected == actual
        }

        fn is_subset(a: Reference, b: Reference) -> bool {
            let a1: Test = a.iter().cloned().collect();
            let b1: Test = b.iter().cloned().collect();
            let actual = a1.is_subset(&b1);
            let expected = a.is_subset(&b);
            expected == actual
        }

        fn contains(a: Reference, b: i64) -> bool {
            let a1: Test = a.iter().cloned().collect();
            let expected = a.contains(&b);
            let actual = a1.contains(&b);
            expected == actual
        }
    }

    bitop_assign_consistent!(Test);
    set_predicate_consistent!(Test);
    bitop_symmetry!(Test);
    bitop_empty!(Test);
}
