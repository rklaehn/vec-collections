//! A data structure for in place modification of vecs.
#![deny(warnings)]
#![deny(missing_docs)]
use std::fmt::Debug;
use std::mem::MaybeUninit;

/// A contiguous chunk of memory that is logically divided into a source and a target part.
///
/// This is using a Vec<T> `v` as storage, but allows an unitialized area inside the Vec!
/// Everything between `t0` and `s1` is uninitialized and must not be dropped!
pub struct FlipBuffer<T> {
    /// the underlying vector, possibly containing some uninitialized values in the middle!
    v: Vec<T>,
    /// the end of the target area
    t1: usize,
    /// the start of the source area
    s0: usize,
}

impl<T: Debug> Debug for FlipBuffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FlipBuffer({:?},{:?})",
            self.target_slice(),
            self.source_slice()
        )
    }
}

/// initializes the source part of this flip buffer with the given vector.
/// The target part is initially empty.
impl<T> From<Vec<T>> for FlipBuffer<T> {
    fn from(value: Vec<T>) -> Self {
        Self { v: value, s0: 0, t1: 0 }
    }
}

impl<T> FlipBuffer<T> {

    /// The current target part as a slice
    pub fn target_slice(&self) -> &[T] {
        &self.v[..self.t1]
    }

    /// The current source part as a slice
    pub fn source_slice(&self) -> &[T] {
        &self.v[self.s0..]
    }

    /// ensure that we have at least `capacity` space. If we have less than that, we will make `gap` space.
    /// if `gap` is less than `capacity`, we will make exactly `capacity` space. But that can be inefficient
    /// when you know that you will need more room later. So typical usage is to provide the maximum you might
    /// need as `gap`.
    /// 
    /// Note that if we have `capacity` space, nothing will be done no matter what the value of `gap` is.
    fn ensure_capacity(&mut self, capacity: usize, gap: usize) {
        // ensure we have space!
        if self.t1 + capacity > self.s0 {
            let capacity = std::cmp::max(gap, capacity);
            // insert missing uninitialized dummy elements before s0
            self.v
                .splice(self.s0..self.s0, Spacer::<T>::sized(capacity));
            // move s0
            self.s0 += capacity;
        }
    }

    /// Take at most `n` elements from `iter` to the target. This will make room for `gap` elements if there is no space
    pub fn target_extend_from_iter<I: Iterator<Item=T>>(&mut self, iter: &mut I, n: usize, gap: usize) {
        if n > 0 {
            self.ensure_capacity(n, gap);
            for _ in 0..n {
                if let Some(value) = iter.next() {
                    self.set(self.t1, value);
                    self.t1 += 1;
                } 
            }
        }
    }

    /// Push a single value to the target. This will make room for `gap` elements if there is no space
    pub fn target_push(&mut self, value: T, gap: usize) {
        // ensure we have space!
        self.ensure_capacity(1, gap);
        self.set(self.t1, value);
        self.t1 += 1;
    }

    /// skip up to `n` elements from source without adding them to the target.
    ///
    /// they will be immediately dropped!
    pub fn skip(&mut self, n: usize) {
        let n = std::cmp::min(n, self.source_slice().len());
        for i in 0..n {
            // this is not a noop but is necessary to call drop!
            let _ = self.get(self.s0 + i);
        }
        self.s0 += n;
    }

    /// take up to `n` elements from source to target.
    ///
    /// If n is larger than the size of the remaining source, this will only copy all remaining elements in source.
    pub fn take(&mut self, n: usize) {
        let n = std::cmp::min(n, self.source_slice().len());
        if self.t1 != self.s0 {
            for i in 0..n {
                self.set(self.t1 + i, self.get(self.s0 + i));
            }
        }
        self.t1 += n;
        self.s0 += n;
    }

    /// Takes the next element from the source, if it exists
    pub fn pop_front(&mut self) -> Option<T> {
        if self.s0 < self.v.len() {
            let old = self.s0;
            self.s0 += 1;
            Some(self.get(old))
        } else {
            None
        }
    }

    fn get(&self, offset: usize) -> T {
        unsafe { std::ptr::read(self.v.as_ptr().add(offset)) }
    }

    fn set(&mut self, offset: usize, value: T) {
        unsafe { std::ptr::write(self.v.as_mut_ptr().add(offset), value) }
    }

    fn drop_source(&mut self) {
        let len = self.v.len();
        // truncate so that just the target part remains
        // this will do nothing to the remaining part of the vector, no drop
        // this is done first so we don't double drop if there is a panic in a drop.
        unsafe {
            self.v.set_len(self.t1);
        }
        // drop all remaining elements of the source, if any
        // in the unlikely case of a panic in drop, we will just stop dropping and thus leak,
        // but not double-drop!
        for i in self.s0..len {
            // I hope this will be just as fast as using std::ptr::drop_in_place...
            let _ = self.get(i);
        }
    }

    /// takes the target part of the flip buffer as a vec and drops the remaining source part, if any
    pub fn into_vec(self) -> Vec<T> {
        let mut r = self;
        r.drop_source();
        let mut t: Vec<T> = Vec::new();
        std::mem::swap(&mut t, &mut r.v);
        t
    }
}

impl<T> Drop for FlipBuffer<T> {
    fn drop(&mut self) {
        // drop the source part.
        // The target part will be dropped normally by the vec itself.
        self.drop_source();
    }
}

/// This thing is evil incarnate. It allows you to create single or multiple T from uninitialized memory
struct Spacer<T>(std::marker::PhantomData<T>);

impl<T> Spacer<T> {
    fn sized(count: usize) -> impl Iterator<Item = T> {
        Self(std::marker::PhantomData).take(count)
    }
    fn single() -> T {
        unsafe { MaybeUninit::uninit().assume_init() }
    }
}

impl<T> Iterator for Spacer<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        Some(Self::single())
    }
}

#[cfg(test)]
mod tests {
    extern crate testdrop;
    use super::*;
    use testdrop::{Item, TestDrop};

    fn everything_dropped<'a, F>(td: &'a TestDrop, n: usize, f: F)
    where
        F: Fn(Vec<Item<'a>>, Vec<Item<'a>>) -> FlipBuffer<Item<'a>>,
    {
        let mut a: Vec<Item> = Vec::new();
        let mut b: Vec<Item> = Vec::new();
        let mut ids: Vec<usize> = Vec::new();
        for _ in 0..n {
            let (id, item) = td.new_item();
            a.push(item);
            ids.push(id);
        }
        for _ in 0..n {
            let (id, item) = td.new_item();
            b.push(item);
            ids.push(id);
        }
        let fb = f(a, b);
        std::mem::drop(fb);
        for id in ids {
            td.assert_drop(id);
        }
    }

    #[test]
    fn drop_just_source() {
        everything_dropped(&TestDrop::new(), 10, |a, _| a.into())
    }

    #[test]
    fn target_push_gap() {
        everything_dropped(&TestDrop::new(), 10, |a, b| {
            let mut res: FlipBuffer<Item> = a.into();
            for x in b.into_iter() {
                res.target_push(x, 100);
            }
            res
        })
    }

    #[test]
    fn source_move_some() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: FlipBuffer<Item> = a.into();
            res.take(3);
            res
        })
    }

    #[test]
    fn source_move_all() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: FlipBuffer<Item> = a.into();
            res.take(10);
            res
        })
    }

    #[test]
    fn source_drop_some() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: FlipBuffer<Item> = a.into();
            res.skip(3);
            res
        })
    }

    #[test]
    fn source_drop_all() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: FlipBuffer<Item> = a.into();
            res.skip(10);
            res
        })
    }

    #[test]
    fn source_pop_some() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: FlipBuffer<Item> = a.into();
            res.pop_front();
            res.pop_front();
            res.pop_front();
            res
        })
    }
}
