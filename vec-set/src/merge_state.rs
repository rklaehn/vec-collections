use crate::array_map::SliceIterator;
use crate::binary_merge::MergeOperation;
use crate::binary_merge::{EarlyOut, MergeState, MergeStateMod, ShortcutMergeOperation};
use flip_buffer::FlipBuffer;
use std::cmp::Ord;
use std::default::Default;
use std::fmt::Debug;

pub(crate) struct UnsafeInPlaceMergeState<A, B> {
    a: FlipBuffer<A>,
    b: std::vec::IntoIter<B>,
}

impl<A, B> UnsafeInPlaceMergeState<A, B> {
    fn new(a: Vec<A>, b: Vec<B>) -> Self {
        Self {
            a: a.into(),
            b: b.into_iter(),
        }
    }
    fn result(self) -> Vec<A> {
        self.a.into()
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

impl<'a, A, B> MergeState<A, B> for UnsafeInPlaceMergeState<A, B> {
    fn a_slice(&self) -> &[A] {
        &self.a.source_slice()
    }
    fn b_slice(&self) -> &[B] {
        self.b.as_slice()
    }
}

impl<'a, T> MergeStateMod<T, T> for UnsafeInPlaceMergeState<T, T> {
    fn move_a(&mut self, n: usize) -> EarlyOut {
        self.a.source_move(n);
        Some(())
    }
    fn skip_a(&mut self, n: usize) -> EarlyOut {
        self.a.source_drop(n);
        Some(())
    }
    fn move_b(&mut self, n: usize) -> EarlyOut {
        let capacity = self.b_slice().len();
        for _ in 0..n {
            if let Some(elem) = self.b.next() {
                self.a.target_push(elem, capacity);
            }
        }
        Some(())
    }
    fn skip_b(&mut self, n: usize) -> EarlyOut {
        for _ in 0..n {
            let _ = self.b.next();
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
    pub fn merge<O: ShortcutMergeOperation<T, T, Self>>(a: &mut Vec<T>, b: &'a [T], o: O) {
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

impl<'a, T> MergeState<T, T> for InPlaceMergeState<'a, T> {
    fn a_slice(&self) -> &[T] {
        &self.a[self.ab..]
    }
    fn b_slice(&self) -> &[T] {
        self.b
    }
}

impl<'a, T: Clone + Default> MergeStateMod<T, T> for InPlaceMergeState<'a, T> {
    fn move_a(&mut self, n: usize) -> EarlyOut {
        if n > 0 {
            if self.ab != self.rn {
                let s = self.ab;
                let t = self.rn;
                for i in 0..n {
                    self.a[t + i] = self.a[s + i].clone();
                }
            }
            self.ab += n;
            self.rn += n;
        }
        Some(())
    }
    fn skip_a(&mut self, n: usize) -> EarlyOut {
        self.ab += n;
        Some(())
    }
    fn move_b(&mut self, n: usize) -> EarlyOut {
        if n > 0 {
            self.ensure_capacity(n);
            let t = self.rn;
            for i in 0..n {
                self.a[t + i] = self.b[i].clone();
            }
            self.skip_b(n)?;
            self.rn += n;
        }
        Some(())
    }
    fn skip_b(&mut self, n: usize) -> EarlyOut {
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

impl<'a, A, B> MergeState<A, B> for BoolOpMergeState<'a, A, B> {
    fn a_slice(&self) -> &[A] {
        self.a.as_slice()
    }
    fn b_slice(&self) -> &[B] {
        self.b.as_slice()
    }
}

impl<'a, A, B> MergeStateMod<A, B> for BoolOpMergeState<'a, A, B> {
    fn move_a(&mut self, n: usize) -> EarlyOut {
        if n > 0 {
            self.r = true;
            None
        } else {
            Some(())
        }
    }
    fn skip_a(&mut self, n: usize) -> EarlyOut {
        self.a.drop_front(n);
        Some(())
    }
    fn move_b(&mut self, n: usize) -> EarlyOut {
        if n > 0 {
            self.r = true;
            None
        } else {
            Some(())
        }
    }
    fn skip_b(&mut self, n: usize) -> EarlyOut {
        self.b.drop_front(n);
        Some(())
    }
}

/// A merge state where we build into a new vector
pub(crate) struct VecMergeState<'a, T> {
    a: SliceIterator<'a, T>,
    b: SliceIterator<'a, T>,
    r: Vec<T>,
}

impl<'a, T: Debug> Debug for VecMergeState<'a, T> {
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

impl<'a, T> VecMergeState<'a, T> {
    pub fn new(a: &'a [T], b: &'a [T], r: Vec<T>) -> Self {
        Self {
            a: SliceIterator(a),
            b: SliceIterator(b),
            r,
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        self.r
    }

    pub fn merge<O: ShortcutMergeOperation<T, T, Self>>(a: &'a [T], b: &'a [T], o: O) -> Vec<T> {
        let t: Vec<T> = Vec::new();
        let mut state = VecMergeState::new(a, b, t);
        o.merge(&mut state);
        state.into_vec()
    }
}

impl<'a, T> MergeState<T, T> for VecMergeState<'a, T> {
    fn a_slice(&self) -> &[T] {
        self.a.as_slice()
    }
    fn b_slice(&self) -> &[T] {
        self.b.as_slice()
    }
}

impl<'a, T: Clone> MergeStateMod<T, T> for VecMergeState<'a, T> {
    fn move_a(&mut self, n: usize) -> EarlyOut {
        self.r.extend_from_slice(self.a.take_front(n));
        Some(())
    }
    fn skip_a(&mut self, n: usize) -> EarlyOut {
        self.a.drop_front(n);
        Some(())
    }
    fn move_b(&mut self, n: usize) -> EarlyOut {
        self.r.extend_from_slice(self.b.take_front(n));
        Some(())
    }
    fn skip_b(&mut self, n: usize) -> EarlyOut {
        self.b.drop_front(n);
        Some(())
    }
}

/// A merge state where we build into a new vector
pub(crate) struct UnsafeSliceMergeState<'a, T> {
    a: &'a [T],
    b: &'a [T],
    r: *mut T,
    r0: usize,
    r1: usize,
}

impl<'a, T> UnsafeSliceMergeState<'a, T> {
    fn merge0<O: ShortcutMergeOperation<T, T, Self>>(
        a: &'a [T],
        b: &'a [T],
        r: *mut T,
        o: O,
    ) -> usize {
        let required = a.len() + b.len();
        let mut state: UnsafeSliceMergeState<'a, T> = UnsafeSliceMergeState {
            a,
            b,
            r,
            r0: 0,
            r1: required,
        };
        o.merge(&mut state);
        debug_assert!(state.r0 == state.r1);
        state.r0
    }

    fn merge<O: ShortcutMergeOperation<T, T, Self>>(
        mut a: Vec<T>,
        mut b: Vec<T>,
        r: &'a mut Vec<T>,
        o: O,
    ) {
        let required = a.len() + b.len();
        let base = r.len();
        r.reserve(required);
        let cap = unsafe {
            // we copy the values into nomansland after the end of r without caring in any way about dropping etc.
            let sa = std::slice::from_raw_parts(a.as_ptr(), a.len());
            let sb = std::slice::from_raw_parts(b.as_ptr(), b.len());
            let cap = UnsafeSliceMergeState::merge0(sa, sb, r.as_mut_ptr().add(base), o);
            // truncate a without dropping
            a.set_len(0);
            // truncate b without dropping
            b.set_len(0);
            // enlarge r to where we know the elements are
            r.set_len(base + required);
            // return cap
            cap
        };
        // drop the elements that have been discarded
        r.truncate(base + cap);
    }
}

impl<'a, T> MergeState<T, T> for UnsafeSliceMergeState<'a, T> {
    fn a_slice(&self) -> &[T] {
        self.a
    }
    fn b_slice(&self) -> &[T] {
        self.b
    }
}

impl<'a, T> MergeStateMod<T, T> for UnsafeSliceMergeState<'a, T> {
    fn move_a(&mut self, n: usize) -> EarlyOut {
        unsafe {
            std::ptr::copy_nonoverlapping(self.a.as_ptr(), self.r.add(self.r0), n);
            self.a = &self.a[n..];
            self.r0 += n;
        }
        Some(())
    }
    fn move_b(&mut self, n: usize) -> EarlyOut {
        unsafe {
            std::ptr::copy_nonoverlapping(self.b.as_ptr(), self.r.add(self.r0), n);
            self.b = &self.b[n..];
            self.r0 += n;
        }
        Some(())
    }
    fn skip_a(&mut self, n: usize) -> EarlyOut {
        unsafe {
            self.r1 -= n;
            std::ptr::copy_nonoverlapping(self.a.as_ptr(), self.r.add(self.r1), n);
            self.a = &self.a[n..];
        }
        Some(())
    }
    fn skip_b(&mut self, n: usize) -> EarlyOut {
        unsafe {
            self.r1 -= n;
            std::ptr::copy_nonoverlapping(self.b.as_ptr(), self.r.add(self.r1), n);
            self.b = &self.b[n..];
        }
        Some(())
    }
}
