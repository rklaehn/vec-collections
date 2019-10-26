use crate::{EarlyOut, InPlaceMergeState, MergeOperation, MergeState, BoolOpMergeState};
use std::fmt::Debug;

struct SetUnionOp();
struct SetIntersectionOp();
struct SetXorOp();
struct SetExceptOp();

impl<'a, T: Ord + Copy + Default, I: MergeState<T> + Debug> MergeOperation<'a, T, I>
    for SetUnionOp
{
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a {}", n);
        m.move_a(n)
        // println!("{:?}\n", m);
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_b {}", n);
        m.move_b(n)
        // println!("{:?}\n", m);
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a 1");
        // println!("skip_b 1");
        m.move_a(1)?;
        m.skip_b(1)
        // println!("{:?}\n", m);
    }
}

impl<'a, T: Ord + Copy + Default, I: MergeState<T> + Debug> MergeOperation<'a, T, I>
    for SetIntersectionOp
{
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a {}", n);
        m.skip_a(n)
        // println!("{:?}\n", m);
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_b {}", n);
        m.skip_b(n)
        // println!("{:?}\n", m);
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a 1");
        // println!("skip_b 1");
        m.move_a(1)?;
        m.skip_b(1)
        // println!("{:?}\n", m);
    }
}

impl<'a, T: Ord + Copy + Default, I: MergeState<T> + Debug> MergeOperation<'a, T, I>
    for SetExceptOp
{
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a {}", n);
        m.move_a(n)
        // println!("{:?}\n", m);
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_b {}", n);
        m.skip_b(n)
        // println!("{:?}\n", m);
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a 1");
        // println!("skip_b 1");
        m.skip_a(1)?;
        m.skip_b(1)
        // println!("{:?}\n", m);
    }
}

impl<'a, T: Ord + Copy + Default, I: MergeState<T> + Debug> MergeOperation<'a, T, I> for SetXorOp {
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a {}", n);
        m.move_a(n)
        // println!("{:?}\n", m);
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_b {}", n);
        m.move_b(n)
        // println!("{:?}\n", m);
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        // println!("{:?}", m);
        // println!("move_a 1");
        // println!("skip_b 1");
        m.skip_a(1)?;
        m.skip_b(1)
        // println!("{:?}\n", m);
    }
}

#[derive(Debug, Clone, Hash)]
struct ArraySet<T>(Vec<T>);

impl<T> ArraySet<T> {
    fn single(value: T) -> Self {
        Self(vec![value])
    }
    fn into_vec(self) -> Vec<T> {
        self.0
    }
    fn as_slice(&self) -> &[T] {
        &self.0
    }
}
impl<T: Ord + Default + Copy + Debug> From<Vec<T>> for ArraySet<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}
impl<T: Ord + Default + Copy + Debug> std::iter::FromIterator<T> for ArraySet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}
impl<T: Ord + Default + Copy + Debug> ArraySet<T> {
    fn from_vec(vec: Vec<T>) -> Self {
        let mut vec = vec;
        vec.sort();
        Self(vec)
    }

    fn contains(&self, value: &T) -> bool {
        self.0.binary_search(value).is_ok()
    }

    fn union_with(&mut self, that: &ArraySet<T>) {
        InPlaceMergeState::merge(&mut self.0, &that.0, SetUnionOp());
    }

    fn intersection_with(&mut self, that: &ArraySet<T>) {
        InPlaceMergeState::merge(&mut self.0, &that.0, SetIntersectionOp());
    }

    fn xor_with(&mut self, that: &ArraySet<T>) {
        InPlaceMergeState::merge(&mut self.0, &that.0, SetXorOp());
    }

    fn except(&mut self, that: &ArraySet<T>) {
        InPlaceMergeState::merge(&mut self.0, &that.0, SetExceptOp());
    }

    fn is_disjoint(&self, that: &ArraySet<T>) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetIntersectionOp())
    }

    fn is_subset(&self, that: &ArraySet<T>) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetExceptOp())
    }

    fn is_superset(&self, that: &ArraySet<T>) -> bool {
        that.is_subset(self)
    }

    fn insert(&mut self, that: T) {
        InPlaceMergeState::merge(&mut self.0, &[that], SetUnionOp());
    }

    fn remove(&mut self, that: &T) {
        InPlaceMergeState::merge(&mut self.0, &[*that], SetExceptOp());
    }
}

// cargo asm abc::array_set::union_u32
pub fn union_u32(a: &mut Vec<u32>, b: &[u32]) {
    InPlaceMergeState::merge(a, b, SetUnionOp())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn intersection_1() {
        let mut a: ArraySet<usize> = vec![0].into();
        let b: ArraySet<usize> = vec![].into();
        a.intersection_with(&b);
        println!("a {:?}", a);
        assert_eq!(a.into_vec(), vec![]);
    }

    quickcheck! {
        fn union(a: BTreeSet<u32>, b: BTreeSet<u32>) -> bool {
            let mut a1: ArraySet<u32> = a.iter().cloned().collect();
            let mut b1: ArraySet<u32> = b.iter().cloned().collect();
            a1.union_with(&b1);
            let expected: Vec<u32> = a.union(&b).cloned().collect();
            let actual: Vec<u32> = a1.into_vec();
            expected == actual
        }

        fn intersection(a: BTreeSet<u32>, b: BTreeSet<u32>) -> bool {
            let mut a1: ArraySet<u32> = a.iter().cloned().collect();
            let mut b1: ArraySet<u32> = b.iter().cloned().collect();
            a1.intersection_with(&b1);
            let expected: Vec<u32> = a.intersection(&b).cloned().collect();
            let actual: Vec<u32> = a1.into_vec();
            expected == actual
        }

        fn is_disjoint(a: BTreeSet<u32>, b: BTreeSet<u32>) -> bool {
            let mut a1: ArraySet<u32> = a.iter().cloned().collect();
            let mut b1: ArraySet<u32> = b.iter().cloned().collect();
            let actual = a1.is_disjoint(&b1);
            let expected = a.is_disjoint(&b);
            expected == actual
        }

        fn is_subset(a: BTreeSet<u32>, b: BTreeSet<u32>) -> bool {
            let mut a1: ArraySet<u32> = a.iter().cloned().collect();
            let mut b1: ArraySet<u32> = b.iter().cloned().collect();
            let actual = a1.is_subset(&b1);
            let expected = a.is_subset(&b);
            expected == actual
        }

        fn except(a: BTreeSet<u32>, b: BTreeSet<u32>) -> bool {
            let mut a1: ArraySet<u32> = a.iter().cloned().collect();
            let mut b1: ArraySet<u32> = b.iter().cloned().collect();
            a1.except(&b1);
            let expected: Vec<u32> = a.difference(&b).cloned().collect();
            let actual: Vec<u32> = a1.into_vec();
            expected == actual
        }

        fn xor(a: BTreeSet<u32>, b: BTreeSet<u32>) -> bool {
            let mut a1: ArraySet<u32> = a.iter().cloned().collect();
            let mut b1: ArraySet<u32> = b.iter().cloned().collect();
            a1.xor_with(&b1);
            let expected: Vec<u32> = a.symmetric_difference(&b).cloned().collect();
            let actual: Vec<u32> = a1.into_vec();
            expected == actual
        }

        fn contains(a: BTreeSet<u32>, b: u32) -> bool {
            let mut a1: ArraySet<u32> = a.iter().cloned().collect();
            let expected = a.contains(&b);
            let actual = a1.contains(&b);
            expected == actual
        }
    }
}
