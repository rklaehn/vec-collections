use crate::binary_merge::{EarlyOut, MergeOperation, MergeStateRead, ShortcutMergeOperation};
use crate::flip_buffer::InPlaceVecBuilder;
use crate::iterators::SliceIterator;
use smallvec::{Array, SmallVec};
use std::cmp::Ord;
use std::default::Default;
use std::fmt::Debug;
use crate::flip_buffer::small_vec_builder::{InPlaceSmallVecBuilder, SmallVecIntoIter};

/// A typical write part for the merge state
pub(crate) trait MergeStateMut<A, B>: MergeStateRead<A, B> {
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

impl<'a, A: Array, B: Array> MergeStateRead<A::Item, B::Item> for SmallVecInPlaceMergeState<A, B> {
    fn a_slice(&self) -> &[A::Item] {
        &self.a.source_slice()
    }
    fn b_slice(&self) -> &[B::Item] {
        self.b.as_slice()
    }
}

impl<'a, A: Array> MergeStateMut<A::Item, A::Item> for SmallVecInPlaceMergeState<A, A> {
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
    pub fn merge_shortcut<O: ShortcutMergeOperation<A::Item, B::Item, Self>>(a: &mut SmallVec<A>, b: SmallVec<B>, o: O) {
        let mut t: SmallVec<A> = Default::default();
        std::mem::swap(a, &mut t);
        let mut state = Self::new(t, b);
        o.merge(&mut state);
        *a = state.result();
    }

    pub fn merge<O: MergeOperation<A::Item, B::Item, Self>>(a: &mut SmallVec<A>, b: SmallVec<B>, o: O) {
        let mut t: SmallVec<A> = Default::default();
        std::mem::swap(a, &mut t);
        let mut state = Self::new(t, b);
        o.merge(&mut state);
        *a = state.result();
    }
}

pub(crate) struct UnsafeInPlaceMergeState<A, B> {
    pub a: InPlaceVecBuilder<A>,
    pub b: std::vec::IntoIter<B>,
}

impl<A, B> UnsafeInPlaceMergeState<A, B> {
    fn new(a: Vec<A>, b: Vec<B>) -> Self {
        Self {
            a: a.into(),
            b: b.into_iter(),
        }
    }
    fn result(self) -> Vec<A> {
        self.a.into_vec()
    }
}

impl<'a, A, B> UnsafeInPlaceMergeState<A, B> {
    pub fn merge_shortcut<O: ShortcutMergeOperation<A, B, Self>>(a: &mut Vec<A>, b: Vec<B>, o: O) {
        let mut t: Vec<A> = Default::default();
        std::mem::swap(a, &mut t);
        let mut state = Self::new(t, b);
        o.merge(&mut state);
        *a = state.result();
    }

    pub fn merge<O: MergeOperation<A, B, Self>>(a: &mut Vec<A>, b: Vec<B>, o: O) {
        let mut t: Vec<A> = Default::default();
        std::mem::swap(a, &mut t);
        let mut state = Self::new(t, b);
        o.merge(&mut state);
        *a = state.result();
    }
}

impl<'a, A, B> MergeStateRead<A, B> for UnsafeInPlaceMergeState<A, B> {
    fn a_slice(&self) -> &[A] {
        &self.a.source_slice()
    }
    fn b_slice(&self) -> &[B] {
        self.b.as_slice()
    }
}

impl<'a, T> MergeStateMut<T, T> for UnsafeInPlaceMergeState<T, T> {
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
    pub fn merge_shortcut<O: ShortcutMergeOperation<T, T, Self>>(a: &mut Vec<T>, b: &'a [T], o: O) {
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

impl<'a, T> MergeStateRead<T, T> for InPlaceMergeState<'a, T> {
    fn a_slice(&self) -> &[T] {
        &self.a[self.ab..]
    }
    fn b_slice(&self) -> &[T] {
        self.b
    }
}

impl<'a, T: Clone + Default> MergeStateMut<T, T> for InPlaceMergeState<'a, T> {
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
    pub fn merge<O: ShortcutMergeOperation<A, B, Self>>(a: &'a [A], b: &'a [B], o: O) -> bool {
        let mut state = Self::new(a, b);
        o.merge(&mut state);
        state.r
    }
}

impl<'a, A, B> MergeStateRead<A, B> for BoolOpMergeState<'a, A, B> {
    fn a_slice(&self) -> &[A] {
        self.a.as_slice()
    }
    fn b_slice(&self) -> &[B] {
        self.b.as_slice()
    }
}

impl<'a, A, B> MergeStateMut<A, B> for BoolOpMergeState<'a, A, B> {
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

    pub fn merge_shortcut<O: ShortcutMergeOperation<A, B, Self>>(
        a: &'a [A],
        b: &'a [B],
        o: O,
    ) -> SmallVec<Arr> {
        let t: SmallVec<Arr> = SmallVec::new();
        let mut state = Self::new(a, b, t);
        o.merge(&mut state);
        state.into_vec()
    }

    pub fn merge<O: MergeOperation<A, B, Self>>(a: &'a [A], b: &'a [B], o: O) -> SmallVec<Arr> {
        let t: SmallVec<Arr> = SmallVec::new();
        let mut state = Self::new(a, b, t);
        o.merge(&mut state);
        state.into_vec()
    }
}

impl<'a, A, B, Arr: Array> MergeStateRead<A, B> for SmallVecMergeState<'a, A, B, Arr> {
    fn a_slice(&self) -> &[A] {
        self.a.as_slice()
    }
    fn b_slice(&self) -> &[B] {
        self.b.as_slice()
    }
}

impl<'a, T: Clone, Arr: Array<Item = T>> MergeStateMut<T, T> for SmallVecMergeState<'a, T, T, Arr> {
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

    pub fn merge_shortcut<O: ShortcutMergeOperation<A, B, Self>>(
        a: &'a [A],
        b: &'a [B],
        o: O,
    ) -> Vec<R> {
        let t: Vec<R> = Vec::new();
        let mut state = VecMergeState::new(a, b, t);
        o.merge(&mut state);
        state.into_vec()
    }

    pub fn merge<O: MergeOperation<A, B, Self>>(a: &'a [A], b: &'a [B], o: O) -> Vec<R> {
        let t: Vec<R> = Vec::new();
        let mut state = VecMergeState::new(a, b, t);
        o.merge(&mut state);
        state.into_vec()
    }
}

impl<'a, A, B, R> MergeStateRead<A, B> for VecMergeState<'a, A, B, R> {
    fn a_slice(&self) -> &[A] {
        self.a.as_slice()
    }
    fn b_slice(&self) -> &[B] {
        self.b.as_slice()
    }
}

impl<'a, T: Clone> MergeStateMut<T, T> for VecMergeState<'a, T, T, T> {
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

/// A merge state where we build into a new vector
pub(crate) struct UnsafeSliceMergeState<T> {
    a: *mut T,
    an: usize,
    b: *mut T,
    bn: usize,
    r: *mut T,
    rn: usize,
}

impl<T> UnsafeSliceMergeState<T> {
    pub fn merge<O: ShortcutMergeOperation<T, T, Self>>(
        v: &mut Vec<T>,
        an: usize,
        bn: usize,
        o: O,
    ) {
        assert!(an + bn <= v.len());
        let len = v.len();
        let base = len - an - bn;
        let rn = an + bn;
        v.reserve(an + bn);
        unsafe {
            v.set_len(base);
            let a = v.as_mut_ptr().add(base);
            let b = v.as_mut_ptr().add(base + an);
            let r = v.as_mut_ptr().add(base + an + bn);
            let mut state = Self {
                a,
                an,
                b,
                bn,
                r,
                rn,
            };
            o.merge(&mut state);
            let copied = rn - state.rn;
            std::ptr::copy_nonoverlapping(r, a, copied);
            v.set_len(base + copied);
        }
    }
}

impl<T> MergeStateRead<T, T> for UnsafeSliceMergeState<T> {
    fn a_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.a, self.an) }
    }
    fn b_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.b, self.bn) }
    }
}

impl<T> MergeStateMut<T, T> for UnsafeSliceMergeState<T> {
    fn advance_a(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            unsafe {
                std::ptr::copy_nonoverlapping(self.a, self.r, n);
                self.a = self.a.add(n);
                self.r = self.r.add(n);
            }
            self.rn -= n;
        } else {
            unsafe {
                for i in 0..n {
                    std::ptr::drop_in_place(self.a.add(i));
                }
                self.a = self.a.add(n);
            }
        }
        self.an -= n;
        Some(())
    }
    fn advance_b(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            unsafe {
                std::ptr::copy_nonoverlapping(self.b, self.r, n);
                self.b = self.b.add(n);
                self.r = self.r.add(n);
            }
            self.rn -= n;
        } else {
            unsafe {
                for i in 0..n {
                    std::ptr::drop_in_place(self.b.add(i));
                }
                self.b = self.b.add(n);
            }
        }
        self.bn -= n;
        Some(())
    }
}
