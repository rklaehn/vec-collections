use crate::binary_merge::*;

struct SetUnionOp();

impl<'a, T: Ord + Copy + Default> Op<'a, T> for SetUnionOp {
    fn from_a(&self, m: &mut BinaryMerge<'a, T>, a0: usize, a1: usize) {
        m.a.copy_from_src(a1 - a0);
    }
    fn from_b(&self, m: &mut BinaryMerge<'a, T>, b0: usize, b1: usize) {
        m.a.copy_from(&m.b[b0..b1], b1 - b0);
    }
    fn collision(&self, m: &mut BinaryMerge<'a, T>, ai: usize, bi: usize) {
        m.a.copy_from_src(1);
    }
}

struct SetIntersectionOp();

impl<'a, T: Ord + Copy + Default> Op<'a, T> for SetIntersectionOp {
    fn from_a(&self, m: &mut BinaryMerge<'a, T>, a0: usize, a1: usize) {
        m.a.drop_from_src(a1 - a0);
    }
    fn from_b(&self, _m: &mut BinaryMerge<'a, T>, b0: usize, b1: usize) {}
    fn collision(&self, m: &mut BinaryMerge<'a, T>, ai: usize, bi: usize) {
        m.a.copy_from_src(1);
    }
}

struct SetXorOp();

impl<'a, T: Ord + Copy + Default> Op<'a, T> for SetXorOp {
    fn from_a(&self, m: &mut BinaryMerge<'a, T>, a0: usize, a1: usize) {
        m.a.copy_from_src(a1 - a0);
    }
    fn from_b(&self, m: &mut BinaryMerge<'a, T>, b0: usize, b1: usize) {
        m.a.copy_from(&m.b[b0..b1], b1 - b0);
    }
    fn collision(&self, m: &mut BinaryMerge<'a, T>, ai: usize, bi: usize) {
        m.a.drop_from_src(1);
    }
}

struct SetExceptOp();

impl<'a, T: Ord + Copy + Default> Op<'a, T> for SetExceptOp {
    fn from_a(&self, m: &mut BinaryMerge<'a, T>, a0: usize, a1: usize) {
        m.a.copy_from_src(a1 - a0);
    }
    fn from_b(&self, m: &mut BinaryMerge<'a, T>, b0: usize, b1: usize) {}
    fn collision(&self, m: &mut BinaryMerge<'a, T>, ai: usize, bi: usize) {
        m.a.drop_from_src(1);
    }
}

impl<'a, T: Ord + Copy + Default> BinaryMerge<'a, T> {
    fn from_a(&mut self, a0: usize, a1: usize) {
        self.a.copy_from_src(a1 - a0);
    }
    fn from_b(&mut self, b0: usize, b1: usize) {
        self.a.copy_from(&self.b[b0..b1], self.b.len() - b0);
    }
    fn collision(&mut self, _ai: usize, _bi: usize) {
        self.a.copy_from_src(1);
    }
    pub fn merge(&mut self) {
        self.merge0(0, self.a.data.len(), 0, self.b.len());
    }
    fn merge0(&mut self, a0: usize, a1: usize, b0: usize, b1: usize) {
        if a0 == a1 {
            self.from_b(b0, b1)
        } else if b0 == b1 {
            self.from_a(a0, a1)
        } else {
            let am: usize = (a0 + a1) / 2;
            match self.b[b0..b1].binary_search(&self.a.src_at(am)) {
                Result::Ok(i) => {
                    let bm = i + b0;
                    // same elements. bm is the index corresponding to am
                    // merge everything below a(am) with everything below the found element
                    self.merge0(a0, am, b0, bm);
                    // add the elements a(am) and b(bm)
                    self.collision(am, bm);
                    // merge everything above a(am) with everything above the found element
                    self.merge0(am + 1, a1, bm + 1, b1);
                }
                Result::Err(i) => {
                    let bi = i + b0;
                    // not found. bi is the insertion point
                    // merge everything below a(am) with everything below the found insertion point
                    self.merge0(a0, am, b0, bi);
                    // add a(am)
                    self.from_a(am, am + 1);
                    // everything above a(am) with everything above the found insertion point
                    self.merge0(am + 1, a1, bi, b1);
                }
            }
        }
    }
}

pub fn merge<T: Ord + Default + Copy>(a: Vec<T>, b: &[T]) -> Vec<T> {
    let mut m = BinaryMerge {
        a: FlipBuffer::new(a),
        b,
    };
    m.merge();
    m.a.result()
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
