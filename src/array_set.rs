use crate::binary_merge::*;

trait MergeState<T> {
    fn a_slice(&self) -> &[T];
    fn b_slice(&self) -> &[T];
    fn r_slice(&self) -> &[T];
    fn move_a(&mut self, n: usize);
    fn skip_a(&mut self, n: usize);
    fn move_b(&mut self, n: usize);
    fn skip_b(&mut self, n: usize);
}

struct InPlaceMergeState<'a, T> {
    a: Vec<T>,
    b: &'a [T],
    // number of result elements
    rn: usize,
    // base of the remaining stuff in a
    ab: usize,
}

impl<'a, T: Copy + Default> InPlaceMergeState<'a, T> {
    fn ensure_capacity(&mut self, required: usize) {
        let rn = self.rn;
        let ab = self.ab;
        let capacity = ab - rn;
        if capacity < required {
            let missing = required - capacity;
            let fill = T::default();
            self.a.splice(ab..ab, std::iter::repeat(fill).take(missing));
            self.ab += missing;
        }
    }
}

impl<'a, T: Copy + Default> MergeState<T> for InPlaceMergeState<'a, T> {
    fn a_slice(&self) -> &[T] {
        let a0 = self.ab;
        let a1 = self.a.len();
        &self.a[a0..a1]
    }
    fn b_slice(&self) -> &[T] {
        self.b
    }
    fn r_slice(&self) -> &[T] {
        &self.a[0..self.rn]
    }
    fn move_a(&mut self, n: usize) {
        if n > 0 {
            if self.ab != self.rn {
                let a0 = self.ab;
                let a1 = a0 + n;
                self.a.as_mut_slice().copy_within(a0..a1, self.rn);
            }
            self.ab += n;
            self.rn += n;
        }
    }
    fn skip_a(&mut self, n: usize) {
        self.ab += n;
    }
    fn move_b(&mut self, n: usize) {
        if n > 0 {
            self.ensure_capacity(n);
            let r0 = self.rn;
            let r1 = self.rn + n;
            self.a[r0..r1].copy_from_slice(&self.b[0..n]);
            self.rn += n;
        }
    }
    fn skip_b(&mut self, n: usize) {
        let b0 = n;
        let b1 = self.b.len();
        self.b = &self.b[b0..b1];
    }
}

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
        SetUnionOp().merge(&mut self.0, &that.0)
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
        let mut a: ArraySet<usize> = vec![].into();
        let b: ArraySet<usize> = vec![0].into();
        a.xor_with(&b);
        assert_eq!(a.into_vec(), vec![0]);
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
