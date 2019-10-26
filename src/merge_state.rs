use crate::{MergeOperation, MergeState, EarlyOut};
use std::cmp::Ord;
use std::default::Default;
use std::fmt::Debug;

pub(crate) struct InPlaceMergeState<'a, T> {
    a: Vec<T>,
    b: &'a [T],
    // number of result elements
    rn: usize,
    // base of the remaining stuff in a
    ab: usize,
}

impl<'a, T: Copy + Default + Debug> Debug for InPlaceMergeState<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "a: {:?}, b: {:?}, r: {:?}",
            self.a_slice(),
            self.b_slice(),
            self.r_slice()
        )
    }
}

impl<'a, T: Copy + Default + Ord> InPlaceMergeState<'a, T> {
    pub fn merge<O: MergeOperation<'a, T, Self>>(a: &mut Vec<T>, b: &'a [T], o: O) {
        let mut t: Vec<T> = Default::default();
        std::mem::swap(a, &mut t);
        let mut state = InPlaceMergeState::new(t, b);
        o.merge(&mut state);
        *a = state.into_vec();
    }
}

impl<'a, T: Copy + Default> InPlaceMergeState<'a, T> {
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

impl<'a, T: Copy + Default> MergeState<T> for InPlaceMergeState<'a, T> {
    fn a_slice(&self) -> &[T] {
        &self.a[self.ab..]
    }
    fn b_slice(&self) -> &[T] {
        self.b
    }
    fn r_slice(&self) -> &[T] {
        &self.a[0..self.rn]
    }
    fn move_a(&mut self, n: usize) -> EarlyOut {
        if n > 0 {
            if self.ab != self.rn {
                let a0 = self.ab;
                let a1 = a0 + n;
                self.a.as_mut_slice().copy_within(a0..a1, self.rn);
            }
            self.ab += n;
            self.rn += n;
        }
        Ok(())
    }
    fn skip_a(&mut self, n: usize) -> EarlyOut {
        self.ab += n;
        Ok(())
    }
    fn move_b(&mut self, n: usize) -> EarlyOut {
        if n > 0 {
            self.ensure_capacity(n);
            self.a[self.rn..self.rn + n].copy_from_slice(&self.b[..n]);
            self.skip_b(n)?;
            self.rn += n;
        }
        Ok(())
    }
    fn skip_b(&mut self, n: usize) -> EarlyOut {
        self.b = &self.b[n..];
        Ok(())
    }
}

pub(crate) struct BoolOpMergeState<'a, T> {
    a: &'a [T],
    b: &'a [T],
    r: bool,
}

impl<'a, T: Copy + Default + Debug> Debug for BoolOpMergeState<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a: {:?}, b: {:?} r: {}", self.a_slice(), self.b_slice(), self.r)
    }
}

impl<'a, T: Copy + Default> BoolOpMergeState<'a, T> {
    pub fn new(a: &'a [T], b: &'a [T]) -> Self {
        Self { a, b, r: false }
    }
}

impl<'a, T: Copy + Default> MergeState<T> for BoolOpMergeState<'a, T> {

    fn a_slice(&self) -> &[T] {
        self.a
    }
    fn b_slice(&self) -> &[T] {
        self.b
    }
    fn r_slice(&self) -> &[T] {
        // dummy
        &self.a[0..0]
    }
    fn move_a(&mut self, n: usize) -> EarlyOut {
        self.r = true;
        Err(())
    }
    fn skip_a(&mut self, n: usize) -> EarlyOut {
        self.a = &self.a[n..];
        self.r = true;
        Ok(())
    }
    fn move_b(&mut self, n: usize) -> EarlyOut {
        self.r = true;
        Err(())
    }
    fn skip_b(&mut self, n: usize) -> EarlyOut {
        self.b = &self.b[n..];
        self.r = true;
        Ok(())
    }
}