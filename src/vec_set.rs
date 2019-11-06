use crate::binary_merge::{EarlyOut, ShortcutMergeOperation};
use crate::dedup::SortAndDedup;
use crate::merge_state::{
    BoolOpMergeState, InPlaceMergeState, MergeStateMut, UnsafeInPlaceMergeState, VecMergeState,
};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::iter::FromIterator;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

struct SetUnionOp;
struct SetIntersectionOp;
struct SetXorOp;
struct SetDiffOpt;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct VecSet<T>(Vec<T>);

impl<T: Ord, I: MergeStateMut<T, T>> ShortcutMergeOperation<T, T, I> for SetUnionOp {
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_a(n)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_b(n)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.move_a(1)?;
        m.skip_b(1)
    }
}

impl<T: Ord, I: MergeStateMut<T, T>> ShortcutMergeOperation<T, T, I> for SetIntersectionOp {
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.skip_a(n)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.skip_b(n)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.move_a(1)?;
        m.skip_b(1)
    }
}

impl<T: Ord, I: MergeStateMut<T, T>> ShortcutMergeOperation<T, T, I> for SetDiffOpt {
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_a(n)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.skip_b(n)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.skip_a(1)?;
        m.skip_b(1)
    }
}

impl<T: Ord, I: MergeStateMut<T, T>> ShortcutMergeOperation<T, T, I> for SetXorOp {
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_a(n)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_b(n)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.skip_a(1)?;
        m.skip_b(1)
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
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.0.iter()
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

impl<T: Ord> BitAnd for VecSet<T> {
    type Output = VecSet<T>;
    fn bitand(mut self, that: Self) -> Self::Output {
        self &= that;
        self
    }
}

impl<T: Ord + Clone> BitOr for &VecSet<T> {
    type Output = VecSet<T>;
    fn bitor(self, that: Self) -> Self::Output {
        VecSet(VecMergeState::merge_shortcut(&self.0, &that.0, SetUnionOp))
    }
}

impl<T: Ord> BitOr for VecSet<T> {
    type Output = VecSet<T>;
    fn bitor(mut self, that: Self) -> Self::Output {
        self |= that;
        self
    }
}

impl<T: Ord + Clone> BitXor for &VecSet<T> {
    type Output = VecSet<T>;
    fn bitxor(self, that: Self) -> Self::Output {
        VecSet(VecMergeState::merge_shortcut(&self.0, &that.0, SetXorOp))
    }
}

impl<T: Ord> BitXor for VecSet<T> {
    type Output = VecSet<T>;
    fn bitxor(mut self, that: Self) -> Self::Output {
        self ^= that;
        self
    }
}

impl<T: Ord + Clone> Sub for &VecSet<T> {
    type Output = VecSet<T>;
    fn sub(self, that: Self) -> Self::Output {
        VecSet(VecMergeState::merge_shortcut(&self.0, &that.0, SetDiffOpt))
    }
}

impl<T: Ord> Sub for VecSet<T> {
    type Output = VecSet<T>;
    fn sub(mut self, that: Self) -> Self::Output {
        self -= that;
        self
    }
}

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
        let iter = iter.into_iter();
        let mut agg = SortAndDedup::<T>::new();
        for x in iter {
            agg.push(x);
        }
        Self::from_vec(agg.result())
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
    InPlaceMergeState::merge(a, b, SetUnionOp)
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::*;
    use std::collections::BTreeSet;

    impl<T: Arbitrary + Ord + Copy + Default + Debug> Arbitrary for VecSet<T> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            VecSet::from_vec(Arbitrary::arbitrary(g))
        }
    }

    fn binary_op(a: &Test, b: &Test, r: &Test, op: impl Fn(bool, bool) -> bool) -> bool {
        let mut samples: Reference = BTreeSet::new();
        samples.extend(a.as_slice().iter().cloned());
        samples.extend(b.as_slice().iter().cloned());
        samples.insert(std::i64::MIN);
        samples
            .iter()
            .all(|e| op(a.contains(e), b.contains(e)) == r.contains(e))
    }

    fn binary_property(a: &Test, b: &Test, r: bool, op: impl Fn(bool, bool) -> bool) -> bool {
        let mut samples: Reference = BTreeSet::new();
        samples.extend(a.as_slice().iter().cloned());
        samples.extend(b.as_slice().iter().cloned());
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

    type Test = VecSet<i64>;
    type Reference = BTreeSet<i64>;

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
}
