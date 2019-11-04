// a set of intervals for countable numbers (integers etc.)
use std::ops::RangeBounds;
use std::ops::Bound;
use std::ops::Bound::*;
use std::cmp::Ordering;
use crate::binary_merge::MergeOperation;
use crate::binary_merge::MergeState;

#[derive(Debug, Clone)]
pub struct CountableIntervalSet<T> {
    below_all: bool,
    boundaries: Vec<T>,
}

impl<T: Clone + Ord> CountableIntervalSet<T> {

    fn from<I: RangeBounds<T>>(rb: I) -> Self {
        Self::from_bounds(
            rb.start_bound(),
            rb.end_bound(),
        )
    }

    fn below(a: &T) -> Self {
        Self::unsafe_new(true, vec![a.clone()])
    }
    fn at_or_above(a: &T) -> Self {
        Self::unsafe_new(false, vec![a.clone()])
    }
    fn from_until(a: &T, b: &T) -> Self {
        if a < b {
            Self::unsafe_new(false, vec![a.clone(), b.clone()])
        } else {
            Self::empty()
        }
    }
    fn from_bounds(a: std::ops::Bound<&T>, b: std::ops::Bound<&T>) -> Self {
        match (a, b) {
            (Unbounded, Unbounded) => Self::all(),
            (Unbounded, Excluded(b)) => Self::below(b),
            (Unbounded, Included(b)) => unimplemented!(),

            (Included(a), Unbounded) => Self::at_or_above(a),
            (Included(a), Excluded(b)) => Self::from_until(a, b),
            (Included(a), Included(b)) => unimplemented!(),

            (Excluded(a), Unbounded) => unimplemented!(),
            (Excluded(a), Excluded(b)) => unimplemented!(),
            (Excluded(a), Included(b)) => unimplemented!(),
        }
    }

    fn union(&self, that: &Self) -> Self {
        Self::unsafe_new(self.below_all | that.below_all, VecMergeState::merge(self, that, UnionOp))
    }

    fn intersection(&self, that: &Self) -> Self {
        Self::unsafe_new(self.below_all | that.below_all, VecMergeState::merge(self, that, IntersectionOp))
    }

    fn xor(&self, that: &Self) -> Self {
        Self::unsafe_new(self.below_all | that.below_all, VecMergeState::merge(self, that, XorOp))
    }
}

impl<T: Ord> CountableIntervalSet<T> {
    fn contains(&self, value: &T) -> bool {
        match self.boundaries.binary_search(value) {
            Ok(index) => self.below_all ^ !is_odd(index),
            Err(index) => self.below_all ^ !is_odd(index - 1),
        }
    }
}

impl<T> CountableIntervalSet<T> {
    fn unsafe_new(below_all: bool, boundaries: Vec<T>) -> Self {
        Self {
            below_all,
            boundaries,
        }
    }
    fn negate(self) -> Self {
        Self::unsafe_new(!self.below_all, self.boundaries)
    }
    fn empty() -> Self {
        Self::unsafe_new(false, Vec::new())
    }
    fn all() -> Self {
        Self::unsafe_new(true, Vec::new())
    }
}

struct VecMergeState<'a, T> {
    ac: bool,
    a: &'a [T],

    bc: bool,
    b: &'a [T],

    r: Vec<T>,
}

fn is_odd(x: usize) -> bool {
    (x & 1) != 0
}

trait MergeStateMut {
    fn advance_both(&mut self, copy: bool);
    fn advance_a(&mut self, n: usize, copy: bool);
    fn advance_b(&mut self, n: usize, copy: bool);
}

impl<'a, T: Clone> VecMergeState<'a, T> {
    fn merge<O: MergeOperation<T, T, Self>>(a: &'a CountableIntervalSet<T>, b: &'a CountableIntervalSet<T>, o: O) -> Vec<T> {
        let mut state = Self {
            ac: a.below_all,
            bc: b.below_all,
            a: a.boundaries.as_slice(),
            b: b.boundaries.as_slice(),
            r: Vec::new(),
        };
        o.merge(&mut state);
        state.r
    }
}

impl<'a, T: Clone> MergeStateMut for VecMergeState<'a, T> {
    fn advance_both(&mut self, copy: bool) {
        self.advance_a(1, copy);
        self.advance_b(1, false);
    } 
    fn advance_a(&mut self, n: usize, copy: bool) {
        if copy {
            self.r.extend_from_slice(&self.a[0..n]);
        }
        self.a = &self.a[n..];
        self.ac ^= is_odd(n);
    }
    fn advance_b(&mut self, n: usize, copy: bool) {
        if copy {
            self.r.extend_from_slice(&self.b[0..n]);
        }
        self.b = &self.b[n..];
        self.bc ^= is_odd(n);
    }
}

impl<'a, T> MergeState<T, T> for VecMergeState<'a, T> {
    /// The remaining data in a
    fn a_slice(&self) -> &[T] {
        &self.a
    }
    /// The remaining data in b
    fn b_slice(&self) -> &[T] {
        &self.b
    }
}

trait Countable {
    fn succ() -> Self;
    fn pred() -> Self;
}

struct UnionOp;
struct IntersectionOp;
struct XorOp;
struct DiffOp;

impl<'a, T: Ord + Clone> MergeOperation<T, T, VecMergeState<'a, T>> for UnionOp {
    fn from_a(&self, m: &mut VecMergeState<'a, T>, n: usize) {
        m.advance_a(n, !m.bc);
    }
    fn from_b(&self, m: &mut VecMergeState<'a, T>, n: usize) {
        m.advance_b(n, !m.ac);
    }
    fn collision(&self, m: &mut VecMergeState<'a, T>) {
        m.advance_both(m.ac == m.bc);
    }
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
}

impl<'a, T: Ord + Clone> MergeOperation<T, T, VecMergeState<'a, T>> for IntersectionOp {
    fn from_a(&self, m: &mut VecMergeState<'a, T>, n: usize) {
        m.advance_a(n, m.bc);
    }
    fn from_b(&self, m: &mut VecMergeState<'a, T>, n: usize) {
        m.advance_b(n, m.ac);
    }
    fn collision(&self, m: &mut VecMergeState<'a, T>) {
        m.advance_both(m.ac == m.bc);
    }
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
}

impl<'a, T: Ord + Clone> MergeOperation<T, T, VecMergeState<'a, T>> for DiffOp {
    fn from_a(&self, m: &mut VecMergeState<'a, T>, n: usize) {
        m.advance_a(n, !m.bc);
    }
    fn from_b(&self, m: &mut VecMergeState<'a, T>, n: usize) {
        m.advance_b(n, m.ac);
    }
    fn collision(&self, m: &mut VecMergeState<'a, T>) {
        m.advance_both(m.ac == !m.bc);
    }
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
}

impl<'a, T: Ord + Clone> MergeOperation<T, T, VecMergeState<'a, T>> for XorOp {
    fn from_a(&self, m: &mut VecMergeState<'a, T>, n: usize) {
        m.advance_a(n, true);
    }
    fn from_b(&self, m: &mut VecMergeState<'a, T>, n: usize) {
        m.advance_b(n, true);
    }
    fn collision(&self, m: &mut VecMergeState<'a, T>) {
        m.advance_both(false);
    }
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Test = CountableIntervalSet<i32>;

    #[test]
    fn smoke_test() {
        let x: Test = Test::from(0..10);
        println!("{:?} {:?} {:?} {:?} {:?}", x, x.contains(&0), x.contains(&1), x.contains(&9), x.contains(&10));

        let y: Test = Test::from(..15);

        let r: Test = x.union(&y);
        let r2: Test = x.intersection(&y);
        let r3: Test = x.xor(&y);

        println!("{:?}", r2);
        println!("{:?}", r3);
    }

    quickcheck! {
        fn create(a: i32, b: i32) -> bool {
            true
        }
    }
}