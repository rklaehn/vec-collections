use crate::binary_merge::{EarlyOut, MergeOperation, MergeStateRead, ShortcutMergeOperation};
use crate::iterators::SliceIterator;
use smallvec::{Array, SmallVec};
use std::cmp::Ord;
use std::default::Default;
use std::fmt::Debug;
use crate::small_vec_builder::{InPlaceSmallVecBuilder, SmallVecIntoIter};

/// A typical write part for the merge state
pub(crate) trait MergeStateMut: MergeStateRead {
    /// Consume n elements of a
    fn advance_a(&mut self, n: usize, take: bool) -> EarlyOut;
    /// Consume n elements of b
    fn advance_b(&mut self, n: usize, take: bool) -> EarlyOut;
}

pub(crate) struct SmallVecInPlaceMergeState<A: Array, B: Array> {
    pub a: InPlaceSmallVecBuilder<A>,
    pub b: SmallVecIntoIter<B>,
}

impl<A: Array, B: Array> SmallVecInPlaceMergeState<A, B> {
    fn new(a: SmallVec<A>, b: SmallVec<B>) -> Self {
        Self {
            a: a.into(),
            b: SmallVecIntoIter::new(b),
        }
    }
    fn result(self) -> SmallVec<A> {
        self.a.into_vec()
    }
}

impl<'a, A: Array, B: Array> MergeStateRead for SmallVecInPlaceMergeState<A, B> {
    type A = A::Item;
    type B = B::Item;
    fn a_slice(&self) -> &[A::Item] {
        &self.a.source_slice()
    }
    fn b_slice(&self) -> &[B::Item] {
        self.b.as_slice()
    }
}

impl<'a, A: Array> MergeStateMut for SmallVecInPlaceMergeState<A, A> {
    fn advance_a(&mut self, n: usize, take: bool) -> EarlyOut {
        self.a.consume(n, take);
        Some(())
    }
    fn advance_b(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.a.extend_from_iter(&mut self.b, n);
        } else {
            for _ in 0..n {
                let _ = self.b.next();
            }
        }
        Some(())
    }
}

impl<'a, A: Array, B: Array> SmallVecInPlaceMergeState<A, B> {
    pub fn merge_shortcut<O: ShortcutMergeOperation<Self>>(a: &mut SmallVec<A>, b: SmallVec<B>, o: O) {
        let mut t: SmallVec<A> = Default::default();
        std::mem::swap(a, &mut t);
        let mut state = Self::new(t, b);
        o.merge(&mut state);
        *a = state.result();
    }

    pub fn merge<O: MergeOperation<Self>>(a: &mut SmallVec<A>, b: SmallVec<B>, o: O) {
        let mut t: SmallVec<A> = Default::default();
        std::mem::swap(a, &mut t);
        let mut state = Self::new(t, b);
        o.merge(&mut state);
        *a = state.result();
    }
}

/// a merge state where the first argument is modified in place
pub(crate) struct InPlaceMergeState<'a, T> {
    a: Vec<T>,
    b: &'a [T],
    // number of result elements
    rn: usize,
    // base of the remaining stuff in a
    ab: usize,
}

impl<'a, T: Debug> Debug for InPlaceMergeState<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "a: {:?}, b: {:?}, r: {:?}",
            self.a_slice(),
            self.b_slice(),
            self.r_slice(),
        )
    }
}

impl<'a, T> InPlaceMergeState<'a, T> {
    fn r_slice(&self) -> &[T] {
        &self.a[..self.rn]
    }
}

impl<'a, T: Clone + Default + Ord> InPlaceMergeState<'a, T> {
    pub fn merge_shortcut<O: ShortcutMergeOperation<Self>>(a: &mut Vec<T>, b: &'a [T], o: O) {
        let mut t: Vec<T> = Default::default();
        std::mem::swap(a, &mut t);
        let mut state = InPlaceMergeState::new(t, b);
        o.merge(&mut state);
        *a = state.into_vec();
    }
}

impl<'a, T: Clone + Default> InPlaceMergeState<'a, T> {
    pub fn new(a: Vec<T>, b: &'a [T]) -> Self {
        Self { a, b, rn: 0, ab: 0 }
    }

    pub fn into_vec(self) -> Vec<T> {
        let mut r = self.a;
        r.truncate(self.rn);
        r
    }

    fn ensure_capacity(&mut self, required: usize) {
        let rn = self.rn;
        let ab = self.ab;
        let capacity = ab - rn;
        if capacity < required {
            // once we need to insert something from b, we pessimistically assume that we need to fit in all of b
            // (for now!)
            let missing = self.b.len();
            let fill = T::default();
            self.a.splice(ab..ab, std::iter::repeat(fill).take(missing));
            self.ab += missing;
        }
    }
}

impl<'a, T> MergeStateRead for InPlaceMergeState<'a, T> {
    type A = T;
    type B = T;
    fn a_slice(&self) -> &[T] {
        &self.a[self.ab..]
    }
    fn b_slice(&self) -> &[T] {
        self.b
    }
}

impl<'a, T: Clone + Default> MergeStateMut for InPlaceMergeState<'a, T> {
    fn advance_a(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            if self.ab != self.rn {
                let s = self.ab;
                let t = self.rn;
                for i in 0..n {
                    self.a[t + i] = self.a[s + i].clone();
                }
            }
            self.rn += n;
        }
        self.ab += n;
        Some(())
    }
    fn advance_b(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.ensure_capacity(n);
            let t = self.rn;
            self.a[t..(t + n)].clone_from_slice(&self.b[..n]);
            self.rn += n;
        }
        self.b = &self.b[n..];
        Some(())
    }
}

/// A merge state where we only track if elements have been produced, and abort as soon as the first element is produced
pub(crate) struct BoolOpMergeState<'a, A, B> {
    a: SliceIterator<'a, A>,
    b: SliceIterator<'a, B>,
    r: bool,
}

impl<'a, A: Debug, B: Debug> Debug for BoolOpMergeState<'a, A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "a: {:?}, b: {:?} r: {}",
            self.a_slice(),
            self.b_slice(),
            self.r
        )
    }
}

impl<'a, A, B> BoolOpMergeState<'a, A, B> {
    pub fn new(a: &'a [A], b: &'a [B]) -> Self {
        Self {
            a: SliceIterator(a),
            b: SliceIterator(b),
            r: false,
        }
    }
}

impl<'a, A, B> BoolOpMergeState<'a, A, B> {
    pub fn merge<O: ShortcutMergeOperation<Self>>(a: &'a [A], b: &'a [B], o: O) -> bool {
        let mut state = Self::new(a, b);
        o.merge(&mut state);
        state.r
    }
}

impl<'a, A, B> MergeStateRead for BoolOpMergeState<'a, A, B> {
    type A = A;
    type B = B;
    fn a_slice(&self) -> &[A] {
        self.a.as_slice()
    }
    fn b_slice(&self) -> &[B] {
        self.b.as_slice()
    }
}

impl<'a, A, B> MergeStateMut for BoolOpMergeState<'a, A, B> {
    fn advance_a(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.r = true;
            None
        } else {
            self.a.drop_front(n);
            Some(())
        }
    }
    fn advance_b(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.r = true;
            None
        } else {
            self.b.drop_front(n);
            Some(())
        }
    }
}

/// A merge state where we build into a new vector
pub(crate) struct SmallVecMergeState<'a, A, B, Arr: Array> {
    pub a: SliceIterator<'a, A>,
    pub b: SliceIterator<'a, B>,
    pub r: SmallVec<Arr>,
}

impl<'a, A: Debug, B: Debug, Arr: Array> Debug for SmallVecMergeState<'a, A, B, Arr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a: {:?}, b: {:?}", self.a_slice(), self.b_slice(),)
    }
}

impl<'a, A, B, Arr: Array> SmallVecMergeState<'a, A, B, Arr> {
    pub fn new(a: &'a [A], b: &'a [B], r: SmallVec<Arr>) -> Self {
        Self {
            a: SliceIterator(a),
            b: SliceIterator(b),
            r,
        }
    }

    pub fn into_vec(self) -> SmallVec<Arr> {
        self.r
    }

    pub fn merge_shortcut<O: ShortcutMergeOperation<Self>>(
        a: &'a [A],
        b: &'a [B],
        o: O,
    ) -> SmallVec<Arr> {
        let t: SmallVec<Arr> = SmallVec::new();
        let mut state = Self::new(a, b, t);
        o.merge(&mut state);
        state.into_vec()
    }

    pub fn merge<O: MergeOperation<Self>>(a: &'a [A], b: &'a [B], o: O) -> SmallVec<Arr> {
        let t: SmallVec<Arr> = SmallVec::new();
        let mut state = Self::new(a, b, t);
        o.merge(&mut state);
        state.into_vec()
    }
}

impl<'a, A, B, Arr: Array> MergeStateRead for SmallVecMergeState<'a, A, B, Arr> {
    type A = A;
    type B = B;
    fn a_slice(&self) -> &[A] {
        self.a.as_slice()
    }
    fn b_slice(&self) -> &[B] {
        self.b.as_slice()
    }
}

impl<'a, T: Clone, Arr: Array<Item = T>> MergeStateMut for SmallVecMergeState<'a, T, T, Arr> {
    fn advance_a(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.r.reserve(n);
            for e in self.a.take_front(n).iter() {
                self.r.push(e.clone())
            }
        } else {
            self.a.drop_front(n);
        }
        Some(())
    }
    fn advance_b(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.r.reserve(n);
            for e in self.b.take_front(n).iter() {
                self.r.push(e.clone())
            }
        } else {
            self.b.drop_front(n);
        }
        Some(())
    }
}

/// A merge state where we build into a new vector
pub(crate) struct VecMergeState<'a, A, B, R> {
    pub a: SliceIterator<'a, A>,
    pub b: SliceIterator<'a, B>,
    pub r: Vec<R>,
}

impl<'a, A: Debug, B: Debug, R: Debug> Debug for VecMergeState<'a, A, B, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "a: {:?}, b: {:?}, r: {:?}",
            self.a_slice(),
            self.b_slice(),
            self.r
        )
    }
}

impl<'a, A, B, R> VecMergeState<'a, A, B, R> {
    pub fn new(a: &'a [A], b: &'a [B], r: Vec<R>) -> Self {
        Self {
            a: SliceIterator(a),
            b: SliceIterator(b),
            r,
        }
    }

    pub fn into_vec(self) -> Vec<R> {
        self.r
    }

    pub fn merge_shortcut<O: ShortcutMergeOperation<Self>>(
        a: &'a [A],
        b: &'a [B],
        o: O,
    ) -> Vec<R> {
        let t: Vec<R> = Vec::new();
        let mut state = VecMergeState::new(a, b, t);
        o.merge(&mut state);
        state.into_vec()
    }

    pub fn merge<O: MergeOperation<Self>>(a: &'a [A], b: &'a [B], o: O) -> Vec<R> {
        let t: Vec<R> = Vec::new();
        let mut state = VecMergeState::new(a, b, t);
        o.merge(&mut state);
        state.into_vec()
    }
}

impl<'a, A, B, R> MergeStateRead for VecMergeState<'a, A, B, R> {
    type A = A;
    type B = B;
    fn a_slice(&self) -> &[A] {
        self.a.as_slice()
    }
    fn b_slice(&self) -> &[B] {
        self.b.as_slice()
    }
}

impl<'a, T: Clone> MergeStateMut for VecMergeState<'a, T, T, T> {
    fn advance_a(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.r.extend_from_slice(self.a.take_front(n));
        } else {
            self.a.drop_front(n);
        }
        Some(())
    }
    fn advance_b(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.r.extend_from_slice(self.b.take_front(n));
        } else {
            self.b.drop_front(n);
        }
        Some(())
    }
}
