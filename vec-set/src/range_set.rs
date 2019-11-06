// a set of non-overlapping ranges
use crate::binary_merge::EarlyOut;
use crate::binary_merge::MergeStateRead;
use crate::binary_merge::ShortcutMergeOperation;
use flip_buffer::FlipBuffer;
use std::cmp::Ordering;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Range, RangeFrom, RangeTo,
    Sub, SubAssign,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeSet<T> {
    below_all: bool,
    boundaries: Vec<T>,
}

impl<T> RangeSet<T> {
    /// note that this is private since it does not check the invariants!
    fn new(below_all: bool, boundaries: Vec<T>) -> Self {
        RangeSet {
            below_all,
            boundaries,
        }
    }
    fn from_range_to(a: T) -> Self {
        Self::new(true, vec![a])
    }
    fn from_range_from(a: T) -> Self {
        Self::new(false, vec![a])
    }
    pub fn empty() -> Self {
        Self::new(false, Vec::new())
    }
    pub fn all() -> Self {
        Self::new(true, Vec::new())
    }
    pub fn constant(value: bool) -> Self {
        Self::new(value, Vec::new())
    }
    pub fn is_empty(&self) -> bool {
        !self.below_all && self.boundaries.is_empty()
    }
    pub fn is_all(&self) -> bool {
        self.below_all && self.boundaries.is_empty()
    }
}

impl<T: Ord> RangeSet<T> {
    fn from_range(a: Range<T>) -> Self {
        if a.start < a.end {
            Self::new(false, vec![a.start, a.end])
        } else {
            Self::empty()
        }
    }

    pub fn is_disjoint(&self, that: &Self) -> bool {
        !((self.below_all & that.below_all) || BoolMergeState::merge(self, that, IntersectionOp))
    }

    pub fn contains(&self, value: &T) -> bool {
        match self.boundaries.binary_search(value) {
            Ok(index) => self.below_all ^ !is_odd(index),
            Err(index) => self.below_all ^ is_odd(index),
        }
    }
}

impl<T: Ord> From<Range<T>> for RangeSet<T> {
    fn from(value: Range<T>) -> Self {
        Self::from_range(value)
    }
}

impl<T: Ord> From<RangeFrom<T>> for RangeSet<T> {
    fn from(value: RangeFrom<T>) -> Self {
        Self::from_range_from(value.start)
    }
}

impl<T: Ord> From<RangeTo<T>> for RangeSet<T> {
    fn from(value: RangeTo<T>) -> Self {
        Self::from_range_to(value.end)
    }
}

impl<T: Ord + Clone> BitAnd for &RangeSet<T> {
    type Output = RangeSet<T>;
    fn bitand(self, that: Self) -> Self::Output {
        Self::Output::new(
            self.below_all & that.below_all,
            VecMergeState::merge(self, that, IntersectionOp),
        )
    }
}

impl<T: Ord> BitAndAssign for RangeSet<T> {
    fn bitand_assign(&mut self, that: Self) {
        InPlaceMergeState::merge(
            &mut self.boundaries,
            self.below_all,
            that.boundaries,
            that.below_all,
            IntersectionOp,
        );
        self.below_all &= that.below_all;
    }
}

impl<T: Ord + Clone> BitOr for &RangeSet<T> {
    type Output = RangeSet<T>;
    fn bitor(self, that: Self) -> Self::Output {
        Self::Output::new(
            self.below_all | that.below_all,
            VecMergeState::merge(self, that, UnionOp),
        )
    }
}

impl<T: Ord> BitOrAssign for RangeSet<T> {
    fn bitor_assign(&mut self, that: Self) {
        InPlaceMergeState::merge(
            &mut self.boundaries,
            self.below_all,
            that.boundaries,
            that.below_all,
            UnionOp,
        );
        self.below_all |= that.below_all;
    }
}

impl<T: Ord + Clone> BitXor for &RangeSet<T> {
    type Output = RangeSet<T>;
    fn bitxor(self, that: Self) -> Self::Output {
        Self::Output::new(
            self.below_all ^ that.below_all,
            VecMergeState::merge(self, that, XorOp),
        )
    }
}

impl<T: Ord> BitXorAssign for RangeSet<T> {
    fn bitxor_assign(&mut self, that: Self) {
        InPlaceMergeState::merge(
            &mut self.boundaries,
            self.below_all,
            that.boundaries,
            that.below_all,
            XorOp,
        );
        self.below_all ^= that.below_all;
    }
}

impl<T: Ord + Clone> Sub for &RangeSet<T> {
    type Output = RangeSet<T>;
    fn sub(self, that: Self) -> Self::Output {
        Self::Output::new(
            self.below_all & !that.below_all,
            VecMergeState::merge(self, that, DiffOp),
        )
    }
}

impl<T: Ord> SubAssign for RangeSet<T> {
    fn sub_assign(&mut self, that: Self) {
        InPlaceMergeState::merge(
            &mut self.boundaries,
            self.below_all,
            that.boundaries,
            that.below_all,
            DiffOp,
        );
        self.below_all &= !that.below_all;
    }
}

impl<T: Ord + Clone> Not for RangeSet<T> {
    type Output = RangeSet<T>;
    fn not(self) -> Self::Output {
        Self::new(!self.below_all, self.boundaries)
    }
}

impl<T: Ord + Clone> Not for &RangeSet<T> {
    type Output = RangeSet<T>;
    fn not(self) -> Self::Output {
        Self::Output::new(!self.below_all, self.boundaries.clone())
    }
}

fn is_odd(x: usize) -> bool {
    (x & 1) != 0
}

trait MergeStateMut<T>: MergeStateRead<T, T> {
    fn advance_both(&mut self, copy: bool) -> EarlyOut;
    fn advance_a(&mut self, n: usize, copy: bool) -> EarlyOut;
    fn advance_b(&mut self, n: usize, copy: bool) -> EarlyOut;
    fn ac(&self) -> bool;
    fn bc(&self) -> bool;
}

struct BoolMergeState<'a, T> {
    ac: bool,
    a: &'a [T],

    bc: bool,
    b: &'a [T],

    r: bool,
}

impl<'a, T> BoolMergeState<'a, T> {
    fn merge<O: ShortcutMergeOperation<T, T, Self>>(
        a: &'a RangeSet<T>,
        b: &'a RangeSet<T>,
        o: O,
    ) -> bool {
        let mut state = Self {
            ac: a.below_all,
            bc: b.below_all,
            a: a.boundaries.as_slice(),
            b: b.boundaries.as_slice(),
            r: false,
        };
        o.merge(&mut state);
        state.r
    }
}

impl<'a, T> MergeStateMut<T> for BoolMergeState<'a, T> {
    fn advance_both(&mut self, copy: bool) -> EarlyOut {
        self.advance_a(1, copy)?;
        self.advance_b(1, false)
    }
    fn advance_a(&mut self, n: usize, copy: bool) -> EarlyOut {
        if copy {
            self.r = true;
            None
        } else {
            self.a = &self.a[n..];
            self.ac ^= is_odd(n);
            Some(())
        }
    }
    fn advance_b(&mut self, n: usize, copy: bool) -> EarlyOut {
        if copy {
            self.r = true;
            None
        } else {
            self.b = &self.b[n..];
            self.bc ^= is_odd(n);
            Some(())
        }
    }
    fn ac(&self) -> bool {
        self.ac
    }
    fn bc(&self) -> bool {
        self.bc
    }
}

impl<'a, T> MergeStateRead<T, T> for BoolMergeState<'a, T> {
    fn a_slice(&self) -> &[T] {
        &self.a
    }
    fn b_slice(&self) -> &[T] {
        &self.b
    }
}

struct VecMergeState<'a, T> {
    ac: bool,
    a: &'a [T],

    bc: bool,
    b: &'a [T],

    r: Vec<T>,
}

impl<'a, T: Clone> VecMergeState<'a, T> {
    fn merge<O: ShortcutMergeOperation<T, T, Self>>(
        a: &'a RangeSet<T>,
        b: &'a RangeSet<T>,
        o: O,
    ) -> Vec<T> {
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

impl<'a, T: Clone> MergeStateMut<T> for VecMergeState<'a, T> {
    fn advance_both(&mut self, copy: bool) -> EarlyOut {
        self.advance_a(1, copy);
        self.advance_b(1, false);
        Some(())
    }
    fn advance_a(&mut self, n: usize, copy: bool) -> EarlyOut {
        if copy {
            self.r.extend_from_slice(&self.a[0..n]);
        }
        self.a = &self.a[n..];
        self.ac ^= is_odd(n);
        Some(())
    }
    fn advance_b(&mut self, n: usize, copy: bool) -> EarlyOut {
        if copy {
            self.r.extend_from_slice(&self.b[0..n]);
        }
        self.b = &self.b[n..];
        self.bc ^= is_odd(n);
        Some(())
    }
    fn ac(&self) -> bool {
        self.ac
    }
    fn bc(&self) -> bool {
        self.bc
    }
}

impl<'a, T> MergeStateRead<T, T> for VecMergeState<'a, T> {
    /// The remaining data in a
    fn a_slice(&self) -> &[T] {
        &self.a
    }
    /// The remaining data in b
    fn b_slice(&self) -> &[T] {
        &self.b
    }
}

struct InPlaceMergeState<T> {
    a: FlipBuffer<T>,
    b: std::vec::IntoIter<T>,
    ac: bool,
    bc: bool,
}

impl<T> InPlaceMergeState<T> {
    pub fn merge<O: ShortcutMergeOperation<T, T, Self>>(
        a: &mut Vec<T>,
        a0: bool,
        b: Vec<T>,
        b0: bool,
        o: O,
    ) {
        let mut t: Vec<T> = Default::default();
        std::mem::swap(a, &mut t);
        let mut state = Self {
            a: t.into(),
            ac: a0,
            b: b.into_iter(),
            bc: b0,
        };
        o.merge(&mut state);
        *a = state.a.into_vec();
    }
}

impl<'a, T> MergeStateRead<T, T> for InPlaceMergeState<T> {
    fn a_slice(&self) -> &[T] {
        self.a.source_slice()
    }
    fn b_slice(&self) -> &[T] {
        self.b.as_slice()
    }
}

impl<'a, T> MergeStateMut<T> for InPlaceMergeState<T> {
    fn advance_both(&mut self, copy: bool) -> EarlyOut {
        self.advance_a(1, copy);
        self.advance_b(1, false);
        Some(())
    }
    fn advance_a(&mut self, n: usize, copy: bool) -> EarlyOut {
        if copy {
            self.a.take(n);
        } else {
            self.a.skip(n);
        }
        self.ac ^= is_odd(n);
        Some(())
    }
    fn advance_b(&mut self, n: usize, copy: bool) -> EarlyOut {
        if copy {
            let capacity = self.b.as_slice().len();
            self.a.target_extend_from_iter(&mut self.b, n, capacity);
        } else {
            for _ in 0..n {
                let _ = self.b.next();
            }
        }
        self.bc ^= is_odd(n);
        Some(())
    }
    fn ac(&self) -> bool {
        self.ac
    }
    fn bc(&self) -> bool {
        self.bc
    }
}

struct UnionOp;
struct IntersectionOp;
struct XorOp;
struct DiffOp;

impl<'a, T: Ord, M: MergeStateMut<T>> ShortcutMergeOperation<T, T, M> for UnionOp {
    fn from_a(&self, m: &mut M, n: usize) -> EarlyOut {
        m.advance_a(n, !m.bc())
    }
    fn from_b(&self, m: &mut M, n: usize) -> EarlyOut {
        m.advance_b(n, !m.ac())
    }
    fn collision(&self, m: &mut M) -> EarlyOut {
        m.advance_both(m.ac() == m.bc())
    }
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
}

impl<'a, T: Ord, M: MergeStateMut<T>> ShortcutMergeOperation<T, T, M> for IntersectionOp {
    fn from_a(&self, m: &mut M, n: usize) -> EarlyOut {
        m.advance_a(n, m.bc())
    }
    fn from_b(&self, m: &mut M, n: usize) -> EarlyOut {
        m.advance_b(n, m.ac())
    }
    fn collision(&self, m: &mut M) -> EarlyOut {
        m.advance_both(m.ac() == m.bc())
    }
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
}

impl<'a, T: Ord, M: MergeStateMut<T>> ShortcutMergeOperation<T, T, M> for DiffOp {
    fn from_a(&self, m: &mut M, n: usize) -> EarlyOut {
        m.advance_a(n, !m.bc())
    }
    fn from_b(&self, m: &mut M, n: usize) -> EarlyOut {
        m.advance_b(n, m.ac())
    }
    fn collision(&self, m: &mut M) -> EarlyOut {
        m.advance_both(m.ac() == !m.bc())
    }
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
}

impl<'a, T: Ord, M: MergeStateMut<T>> ShortcutMergeOperation<T, T, M> for XorOp {
    fn from_a(&self, m: &mut M, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut M, n: usize) -> EarlyOut {
        m.advance_b(n, true)
    }
    fn collision(&self, m: &mut M) -> EarlyOut {
        m.advance_both(false)
    }
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_macros::*;
    use quickcheck::*;
    use std::collections::BTreeSet;
    use std::ops::Range;

    type Elem = i32;
    type Ref = (Vec<Range<Elem>>, bool);
    type Test = RangeSet<i32>;

    fn union_contains(x: &Ref, elem: &Elem) -> bool {
        let (ranges, below_all) = x;
        ranges.iter().any(|range| range.contains(&elem)) ^ below_all
    }

    fn intersection_contains(x: &Ref, elem: &Elem) -> bool {
        let (ranges, below_all) = x;
        ranges.iter().all(|range| range.contains(&elem)) ^ below_all
    }

    fn sample(x: &Ref) -> Vec<Elem> {
        let (ranges, _) = x;
        let mut res: Vec<Elem> = Vec::new();
        res.push(std::i32::MIN);
        for range in ranges {
            res.push(range.start);
            res.push(range.start + 1);
            res.push(range.end - 1);
            res.push(range.end);
        }
        res.sort();
        res.dedup();
        res
    }

    fn to_test(r: &Ref) -> Test {
        let (ranges, below_all) = r;
        let mut res = RangeSet::empty();
        for range in ranges {
            res = res.bitor(&RangeSet::from_range(range.clone()));
            let mut res2 = res.clone();
            res2 |= RangeSet::from_range(range.clone());
            assert_eq!(res, res2);
        }
        if *below_all {
            res = !res;
        }
        res
    }

    #[test]
    fn smoke_test() {
        let x: Test = Test::from(0..10);
        println!(
            "{:?} {:?} {:?} {:?} {:?}",
            x,
            x.contains(&0),
            x.contains(&1),
            x.contains(&9),
            x.contains(&10)
        );

        let y: Test = Test::from(..10);
        let z: Test = Test::from(20..);

        let r: Test = x.bitor(&y);
        let r2: Test = x.bitand(&y);
        let r3: Test = x.bitxor(&y);
        let r4 = y.is_disjoint(&z);
        let r5 = y.bitand(&z);

        println!("{:?}", r2);
        println!("{:?}", r3);
        println!("{:?} {:?}", r4, r5);
    }

    impl<T: Arbitrary + Ord> Arbitrary for RangeSet<T> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut boundaries: Vec<T> = Arbitrary::arbitrary(g);
            let below_all: bool = Arbitrary::arbitrary(g);
            // boundaries.truncate(2);
            boundaries.sort();
            boundaries.dedup();
            Self::new(below_all, boundaries)
        }
    }

    bitop_assign_consistent!(Test);
    bitop_symmetry!(Test);
    empty_neutral!(Test);
    all_neutral!(Test);
}
