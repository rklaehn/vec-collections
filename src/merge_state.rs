use crate::{EarlyOut, MergeOperation, MergeState, MergeStateRead};
use std::cmp::Ord;
use std::default::Default;
use std::fmt::Debug;

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
    pub fn merge<O: MergeOperation<'a, T, T, Self>>(a: &mut Vec<T>, b: &'a [T], o: O) {
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

impl<'a, T: Clone + Default> MergeState<T, T> for InPlaceMergeState<'a, T> {
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
pub(crate) struct BoolOpMergeState<'a, T> {
    a: &'a [T],
    b: &'a [T],
    r: bool,
}

impl<'a, T: Debug> Debug for BoolOpMergeState<'a, T> {
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

impl<'a, T> BoolOpMergeState<'a, T> {
    pub fn new(a: &'a [T], b: &'a [T]) -> Self {
        Self { a, b, r: false }
    }
}

impl<'a, T> BoolOpMergeState<'a, T> {
    pub fn merge<O: MergeOperation<'a, T, T, Self>>(a: &'a [T], b: &'a [T], o: O) -> bool {
        let mut state = BoolOpMergeState::new(a, b);
        o.merge(&mut state);
        state.r
    }
}

impl<'a, T> MergeStateRead<T, T> for BoolOpMergeState<'a, T> {
    fn a_slice(&self) -> &[T] {
        self.a
    }
    fn b_slice(&self) -> &[T] {
        self.b
    }
}

impl<'a, T> MergeState<T, T> for BoolOpMergeState<'a, T> {
    fn move_a(&mut self, n: usize) -> EarlyOut {
        if n > 0 {
            self.r = true;
            None
        } else {
            Some(())
        }
    }
    fn skip_a(&mut self, n: usize) -> EarlyOut {
        self.a = &self.a[n..];
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
        self.b = &self.b[n..];
        Some(())
    }
}

/// A merge state where we build into a new vector
pub(crate) struct VecMergeState<'a, T> {
    a: &'a [T],
    b: &'a [T],
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

impl<'a, T: Clone> VecMergeState<'a, T> {
    pub fn new(a: &'a [T], b: &'a [T], r: Vec<T>) -> Self {
        Self { a, b, r }
    }

    pub fn into_vec(self) -> Vec<T> {
        self.r
    }

    pub fn merge<O: MergeOperation<'a, T, T, Self>>(a: &'a [T], b: &'a [T], o: O) -> Vec<T> {
        let t: Vec<T> = Vec::new();
        let mut state = VecMergeState::new(a, b, t);
        o.merge(&mut state);
        state.into_vec()
    }
}

impl<'a, T> MergeStateRead<T, T> for VecMergeState<'a, T> {
    fn a_slice(&self) -> &[T] {
        self.a
    }
    fn b_slice(&self) -> &[T] {
        self.b
    }
}

impl<'a, T: Clone> MergeState<T, T> for VecMergeState<'a, T> {
    fn move_a(&mut self, n: usize) -> EarlyOut {
        let (c, r) = self.a.split_at(n);
        self.r.extend_from_slice(c);
        self.a = r;
        Some(())
    }
    fn skip_a(&mut self, n: usize) -> EarlyOut {
        self.a = &self.a[n..];
        Some(())
    }
    fn move_b(&mut self, n: usize) -> EarlyOut {
        let (c, r) = self.b.split_at(n);
        self.r.extend_from_slice(c);
        self.b = r;
        Some(())
    }
    fn skip_b(&mut self, n: usize) -> EarlyOut {
        self.b = &self.b[n..];
        Some(())
    }
}
