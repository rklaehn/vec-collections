use crate::binary_merge::{EarlyOut, ShortcutMergeOperation};
use crate::dedup::sort_and_dedup;
use crate::iterators::SortedIter;
use crate::merge_state::{
    BoolOpMergeState, InPlaceMergeState, MergeStateMut, SmallVecMergeState,
    UnsafeInPlaceMergeState, UnsafeSliceMergeState, VecMergeState, SmallVecInPlaceMergeState,
};
use smallvec::{Array, SmallVec};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

struct SetUnionOp;
struct SetIntersectionOp;
struct SetXorOp;
struct SetDiffOpt;

#[derive(Debug, Hash, Clone, PartialEq, Eq, Default)]
pub struct VecSet2<T, A: Array<Item = T> = [T; 2]>(SmallVec<A>, PhantomData<T>);

impl<T, A: Array<Item = T>> VecSet2<T, A> {
    fn new(a: SmallVec<A>) -> Self {
        Self(a, PhantomData)
    }
    pub fn singleton(value: T) -> Self {
        let mut res = SmallVec::new();
        res.push(value);
        Self(res, PhantomData)
    }
    pub fn empty() -> Self {
        Self::new(SmallVec::new())
    }
    /// An iterator that returns the items of this vec set in sorted order
    pub fn iter(&self) -> SortedIter<std::slice::Iter<T>> {
        SortedIter::new(self.0.iter())
    }
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: Ord, A: Array<Item = T>> VecSet2<T, A> {
    pub fn insert(&mut self, that: T) {
        match self.0.binary_search(&that) {
            Ok(index) => self.0[index] = that,
            Err(index) => self.0.insert(index, that),
        }
    }

    pub fn remove(&mut self, that: &T) {
        if let Ok(index) = self.0.binary_search(&that) {
            self.0.remove(index);
        };
    }

    pub fn is_disjoint(&self, that: &Self) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetIntersectionOp)
    }

    pub fn is_subset(&self, that: &Self) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetDiffOpt)
    }

    pub fn is_superset(&self, that: &Self) -> bool {
        that.is_subset(self)
    }

    pub fn contains(&self, value: &T) -> bool {
        self.0.binary_search(value).is_ok()
    }

    fn from_vec(vec: Vec<T>) -> Self {
        let mut vec = vec;
        vec.sort();
        vec.dedup();
        Self::new(SmallVec::from_vec(vec))
    }
}

impl<A: Array<Item = T>, T> Into<Vec<T>> for VecSet2<T, A> {
    fn into(self) -> Vec<T> {
        self.0.into_vec()
    }
}

impl<T: Ord + Clone, Arr: Array<Item = T>> BitAnd for &VecSet2<T, Arr> {
    type Output = VecSet2<T, Arr>;
    fn bitand(self, that: Self) -> Self::Output {
        Self::Output::new(SmallVecMergeState::merge_shortcut(
            &self.0,
            &that.0,
            SetIntersectionOp,
        ))
    }
}

impl<T: Ord + Clone, Arr: Array<Item = T>> BitOr for &VecSet2<T, Arr> {
    type Output = VecSet2<T, Arr>;
    fn bitor(self, that: Self) -> Self::Output {
        Self::Output::new(SmallVecMergeState::merge_shortcut(
            &self.0, &that.0, SetUnionOp,
        ))
    }
}

impl<T: Ord + Clone, Arr: Array<Item = T>> BitXor for &VecSet2<T, Arr> {
    type Output = VecSet2<T, Arr>;
    fn bitxor(self, that: Self) -> Self::Output {
        Self::Output::new(SmallVecMergeState::merge_shortcut(
            &self.0, &that.0, SetXorOp,
        ))
    }
}

impl<T: Ord + Clone, Arr: Array<Item = T>> Sub for &VecSet2<T, Arr> {
    type Output = VecSet2<T, Arr>;
    fn sub(self, that: Self) -> Self::Output {
        Self::Output::new(SmallVecMergeState::merge_shortcut(
            &self.0, &that.0, SetDiffOpt,
        ))
    }
}

impl<T: Ord> BitAndAssign for VecSet2<T> {
    fn bitand_assign(&mut self, that: Self) {
        SmallVecInPlaceMergeState::merge_shortcut(&mut self.0, that.0, SetIntersectionOp);
    }
}

impl<T: Ord> BitOrAssign for VecSet2<T> {
    fn bitor_assign(&mut self, that: Self) {
        SmallVecInPlaceMergeState::merge_shortcut(&mut self.0, that.0, SetUnionOp);
    }
}

impl<T: Ord> BitXorAssign for VecSet2<T> {
    fn bitxor_assign(&mut self, that: Self) {
        SmallVecInPlaceMergeState::merge_shortcut(&mut self.0, that.0, SetXorOp);
    }
}

impl<T: Ord> SubAssign for VecSet2<T> {
    fn sub_assign(&mut self, that: Self) {
        SmallVecInPlaceMergeState::merge_shortcut(&mut self.0, that.0, SetDiffOpt);
    }
}

impl<T: Ord> From<Vec<T>> for VecSet2<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}

impl<T: Ord> From<BTreeSet<T>> for VecSet2<T> {
    fn from(value: BTreeSet<T>) -> Self {
        Self::new(value.into_iter().collect())
    }
}

impl<T: Ord> FromIterator<T> for VecSet2<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(sort_and_dedup(iter.into_iter()))
    }
}

impl<T: Ord> Extend<T> for VecSet2<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        *self &= Self::from_iter(iter);
    }
}

impl<'a, T: 'a + Ord + Copy> Extend<&'a T> for VecSet2<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned())
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct VecSet<T>(Vec<T>);

impl<T: Ord, I: MergeStateMut<T, T>> ShortcutMergeOperation<T, T, I> for SetUnionOp {
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

impl<T: Ord, I: MergeStateMut<T, T>> ShortcutMergeOperation<T, T, I> for SetIntersectionOp {
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

impl<T: Ord, I: MergeStateMut<T, T>> ShortcutMergeOperation<T, T, I> for SetDiffOpt {
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

impl<T: Ord, I: MergeStateMut<T, T>> ShortcutMergeOperation<T, T, I> for SetXorOp {
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

impl<T: Debug> Debug for VecSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.0.iter()).finish()
    }
}

impl<T> Into<Vec<T>> for VecSet<T> {
    fn into(self) -> Vec<T> {
        self.0
    }
}

impl<T> VecSet<T> {
    pub fn singleton(value: T) -> Self {
        Self(vec![value])
    }
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn empty() -> Self {
        Self(Vec::new())
    }
    /// An iterator that returns the items of this vec set in sorted order
    pub fn iter(&self) -> SortedIter<std::slice::Iter<T>> {
        SortedIter::new(self.0.iter())
    }
    pub fn retain<F: FnMut(&T) -> bool>(&mut self, f: F) {
        self.0.retain(f)
    }
}

impl<T> Default for VecSet<T> {
    fn default() -> Self {
        VecSet::empty()
    }
}

impl<T: Ord> BitAndAssign for VecSet<T> {
    fn bitand_assign(&mut self, that: Self) {
        UnsafeInPlaceMergeState::merge_shortcut(&mut self.0, that.0, SetIntersectionOp);
    }
}

impl<T: Ord> BitOrAssign for VecSet<T> {
    fn bitor_assign(&mut self, that: Self) {
        UnsafeInPlaceMergeState::merge_shortcut(&mut self.0, that.0, SetUnionOp);
    }
}

impl<T: Ord> BitXorAssign for VecSet<T> {
    fn bitxor_assign(&mut self, that: Self) {
        UnsafeInPlaceMergeState::merge_shortcut(&mut self.0, that.0, SetXorOp);
    }
}

impl<T: Ord> SubAssign for VecSet<T> {
    fn sub_assign(&mut self, that: Self) {
        UnsafeInPlaceMergeState::merge_shortcut(&mut self.0, that.0, SetDiffOpt);
    }
}

impl<T: Ord + Clone> BitAnd for &VecSet<T> {
    type Output = VecSet<T>;
    fn bitand(self, that: Self) -> Self::Output {
        VecSet(VecMergeState::merge_shortcut(
            &self.0,
            &that.0,
            SetIntersectionOp,
        ))
    }
}

// impl<T: Ord> BitAnd for VecSet<T> {
//     type Output = VecSet<T>;
//     fn bitand(mut self, that: Self) -> Self::Output {
//         self &= that;
//         self
//     }
// }

impl<T: Ord + Clone> BitOr for &VecSet<T> {
    type Output = VecSet<T>;
    fn bitor(self, that: Self) -> Self::Output {
        VecSet(VecMergeState::merge_shortcut(&self.0, &that.0, SetUnionOp))
    }
}

// impl<T: Ord> BitOr for VecSet<T> {
//     type Output = VecSet<T>;
//     fn bitor(mut self, that: Self) -> Self::Output {
//         self |= that;
//         self
//     }
// }

impl<T: Ord + Clone> BitXor for &VecSet<T> {
    type Output = VecSet<T>;
    fn bitxor(self, that: Self) -> Self::Output {
        VecSet(VecMergeState::merge_shortcut(&self.0, &that.0, SetXorOp))
    }
}

// impl<T: Ord> BitXor for VecSet<T> {
//     type Output = VecSet<T>;
//     fn bitxor(mut self, that: Self) -> Self::Output {
//         self ^= that;
//         self
//     }
// }

impl<T: Ord + Clone> Sub for &VecSet<T> {
    type Output = VecSet<T>;
    fn sub(self, that: Self) -> Self::Output {
        VecSet(VecMergeState::merge_shortcut(&self.0, &that.0, SetDiffOpt))
    }
}

// impl<T: Ord> Sub for VecSet<T> {
//     type Output = VecSet<T>;
//     fn sub(mut self, that: Self) -> Self::Output {
//         self -= that;
//         self
//     }
// }

impl<T: Ord> From<Vec<T>> for VecSet<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}

impl<T: Ord> From<BTreeSet<T>> for VecSet<T> {
    fn from(value: BTreeSet<T>) -> Self {
        Self(value.into_iter().collect())
    }
}

impl<T: Ord> FromIterator<T> for VecSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(sort_and_dedup(iter.into_iter()))
    }
}

impl<T: Ord> Extend<T> for VecSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        *self &= Self::from_iter(iter);
    }
}

impl<'a, T: 'a + Ord + Copy> Extend<&'a T> for VecSet<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned())
    }
}

impl<T: Ord> VecSet<T> {
    pub fn insert(&mut self, that: T) {
        match self.0.binary_search(&that) {
            Ok(index) => self.0[index] = that,
            Err(index) => self.0.insert(index, that),
        }
    }

    pub fn remove(&mut self, that: &T) {
        if let Ok(index) = self.0.binary_search(&that) {
            self.0.remove(index);
        };
    }

    pub fn is_disjoint(&self, that: &VecSet<T>) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetIntersectionOp)
    }

    pub fn is_subset(&self, that: &VecSet<T>) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetDiffOpt)
    }

    pub fn is_superset(&self, that: &VecSet<T>) -> bool {
        that.is_subset(self)
    }
    pub fn contains(&self, value: &T) -> bool {
        self.0.binary_search(value).is_ok()
    }

    fn from_vec(vec: Vec<T>) -> Self {
        let mut vec = vec;
        vec.sort();
        vec.dedup();
        Self(vec)
    }
}

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

// cargo asm vec_set::array_set::union_u32
pub fn union_u32(a: &mut Vec<u32>, b: &[u32]) {
    InPlaceMergeState::merge_shortcut(a, b, SetUnionOp)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::obey::*;
    use num_traits::PrimInt;
    use quickcheck::*;
    use std::collections::BTreeSet;

    impl<T: Arbitrary + Ord + Copy + Default + Debug> Arbitrary for VecSet<T> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            VecSet::from_vec(Arbitrary::arbitrary(g))
        }
    }

    impl<E: PrimInt> TestSamples<E, bool> for VecSet<E> {
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

    type Test = VecSet<i64>;
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

    #[test]
    fn full_in_place_merge() {
        let mut v = vec![1, 3, 5, 7, 2, 3, 7, 8];
        UnsafeSliceMergeState::merge(&mut v, 4, 4, SetXorOp);
        v.shrink_to_fit();
        println!("{:?} {}", v, v.capacity());
    }
}

#[cfg(test)]
mod test2 {
    use super::*;
    use crate::obey::*;
    use num_traits::PrimInt;
    use quickcheck::*;
    use std::collections::BTreeSet;

    impl<T: Arbitrary + Ord + Copy + Default + Debug> Arbitrary for VecSet2<T> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Self::from_vec(Arbitrary::arbitrary(g))
        }
    }

    impl<E: PrimInt> TestSamples<E, bool> for VecSet2<E> {
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

    type Test = VecSet2<i64>;
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

    // bitop_assign_consistent!(Test);
    set_predicate_consistent!(Test);
    bitop_symmetry!(Test);
    bitop_empty!(Test);

    #[test]
    fn full_in_place_merge() {
        let mut v = vec![1, 3, 5, 7, 2, 3, 7, 8];
        UnsafeSliceMergeState::merge(&mut v, 4, 4, SetXorOp);
        v.shrink_to_fit();
        println!("{:?} {}", v, v.capacity());
    }
}
