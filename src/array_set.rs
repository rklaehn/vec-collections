use crate::binary_merge::*;

struct SetUnionOp();

impl<'a, T: Ord + Copy + Default> Op<'a, T> for SetUnionOp {
    fn from_a(&self, m: &mut BinaryMerge<'a, T>, n: usize) {
        m.a.copy_from_src(n);
    }
    fn from_b(&self, m: &mut BinaryMerge<'a, T>, b0: usize, b1: usize) {
        m.a.copy_from(&m.b[b0..b1], b1 - b0);
    }
    fn collision(&self, m: &mut BinaryMerge<'a, T>) {
        m.a.copy_from_src(1);
    }
}

impl<'a, T: Ord + Copy + Default, I: MergeState<T>> Op2<'a, T, I> for SetUnionOp {
    fn from_a(&self, m: &mut I, n: usize) {
        println!("move_a {}", n);
        m.move_a(n);
    }
    fn from_b(&self, m: &mut I, n: usize) {
        println!("move_b {}", n);
        m.move_b(n);
    }
    fn collision(&self, m: &mut I) {
        println!("move_a 1");
        println!("skip_b 1");
        m.move_a(1);
        m.skip_b(1);
    }
}

struct SetIntersectionOp();

impl<'a, T: Ord + Copy + Default> Op<'a, T> for SetIntersectionOp {
    fn from_a(&self, m: &mut BinaryMerge<'a, T>, n: usize) {
        m.a.drop_from_src(n);
    }
    fn from_b(&self, _m: &mut BinaryMerge<'a, T>, b0: usize, b1: usize) {}
    fn collision(&self, m: &mut BinaryMerge<'a, T>) {
        m.a.copy_from_src(1);
    }
}

struct SetXorOp();

impl<'a, T: Ord + Copy + Default> Op<'a, T> for SetXorOp {
    fn from_a(&self, m: &mut BinaryMerge<'a, T>, n: usize) {
        m.a.copy_from_src(n);
    }
    fn from_b(&self, m: &mut BinaryMerge<'a, T>, b0: usize, b1: usize) {
        m.a.copy_from(&m.b[b0..b1], b1 - b0);
    }
    fn collision(&self, m: &mut BinaryMerge<'a, T>) {
        m.a.drop_from_src(1);
    }
}

struct SetExceptOp();

impl<'a, T: Ord + Copy + Default> Op<'a, T> for SetExceptOp {
    fn from_a(&self, m: &mut BinaryMerge<'a, T>, n: usize) {
        m.a.copy_from_src(n);
    }
    fn from_b(&self, m: &mut BinaryMerge<'a, T>, b0: usize, b1: usize) {}
    fn collision(&self, m: &mut BinaryMerge<'a, T>) {
        m.a.drop_from_src(1);
    }
}

#[derive(Debug, Clone)]
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
impl<T: Ord + Default + Copy> From<Vec<T>> for ArraySet<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}
impl<T: Ord + Default + Copy> std::iter::FromIterator<T> for ArraySet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}
impl<T: Ord + Default + Copy> ArraySet<T> {
    fn from_vec(vec: Vec<T>) -> Self {
        let mut vec = vec;
        vec.sort();
        Self(vec)
    }

    fn union_with(&mut self, that: &ArraySet<T>) {
        InPlaceMergeState::merge(&mut self.0, &that.0, SetUnionOp());
    }

    fn intersection_with(&mut self, that: &ArraySet<T>) {
        SetIntersectionOp().merge(&mut self.0, &that.0)
    }

    fn xor_with(&mut self, that: &ArraySet<T>) {
        SetXorOp().merge(&mut self.0, &that.0)
    }

    fn except(&mut self, that: &ArraySet<T>) {
        SetExceptOp().merge(&mut self.0, &that.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn union_1() {
        let mut a: ArraySet<usize> = vec![1].into();
        let b: ArraySet<usize> = vec![0,2].into();
        a.union_with(&b);
        assert_eq!(a.into_vec(), vec![0,1,2]);
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
    }
}
