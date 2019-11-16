//! A data structure for in place modification of vecs.
// #![deny(warnings)]
#![deny(missing_docs)]
use std::cell::RefCell;
use std::fmt::Debug;

/// A contiguous chunk of memory that is logically divided into a source and a target part.
/// This can be used to build a `Vec<T>` while reusing elements from an existing `Vec<T>` in place.
///
/// This is using a `Vec<T>` as storage and divides it logically into a source and target part.
/// You can take or drop elements from the source part, and append elements to the target part.
pub struct InPlaceVecBuilder<T> {
    /// the underlying vector, possibly containing some uninitialized values in the middle!
    v: Vec<T>,
    /// the end of the target area
    t1: usize,
    /// the start of the source area
    s0: usize,
}

impl<T> Debug for InPlaceVecBuilder<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let InPlaceVecBuilder { s0, t1, v } = self;
        let s1 = v.len();
        let cap = v.capacity();
        write!(f, "InPlaceVecBuilder(0..{},{}..{},{})", t1, s0, s1, cap)
    }
}

/// initializes the source part of this flip buffer with the given vector.
/// The target part is initially empty.
impl<T> From<Vec<T>> for InPlaceVecBuilder<T> {
    fn from(value: Vec<T>) -> Self {
        InPlaceVecBuilder {
            v: value,
            s0: 0,
            t1: 0,
        }
    }
}

impl<T> InPlaceVecBuilder<T> {
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
            let s0 = self.s0;
            let s1 = self.v.len();
            // reserve_exact because we assume that gap is the worst case that we are going to need
            self.v.reserve_exact(capacity);
            // just move source to the end without any concern about dropping
            unsafe {
                let src = self.v.as_ptr().add(s0);
                let tgt = self.v.as_mut_ptr().add(s0 + capacity);
                std::ptr::copy(src, tgt, s1 - s0);
                self.v.set_len(self.v.capacity());
            }
            // move s0
            self.s0 += capacity;
        }
    }

    /// Take at most `n` elements from `iter` to the target. This will make room for `gap` elements if there is no space
    pub fn extend_from_iter<I: Iterator<Item = T>>(&mut self, iter: &mut I, n: usize, gap: usize) {
        if n > 0 {
            self.ensure_capacity(n, gap);
            for _ in 0..n {
                if let Some(value) = iter.next() {
                    self.push_unsafe(value)
                }
            }
        }
    }

    /// Push a single value to the target. This will make room for `gap` elements if there is no space
    pub fn push(&mut self, value: T, gap: usize) {
        // ensure we have space!
        self.ensure_capacity(1, gap);
        self.push_unsafe(value);
    }

    fn push_unsafe(&mut self, value: T) {
        unsafe { std::ptr::write(self.v.as_mut_ptr().add(self.t1), value) }
        self.t1 += 1;
    }

    /// Skip up to `n` elements from source without adding them to the target.
    /// They will be immediately dropped!
    pub fn skip(&mut self, n: usize) {
        let n = std::cmp::min(n, self.source_slice().len());
        let v = self.v.as_mut_ptr();
        for _ in 0..n {
            unsafe {
                self.s0 += 1;
                std::ptr::drop_in_place(v.add(self.s0 - 1));
            }
        }
    }

    /// Take up to `n` elements from source to target.
    /// If n is larger than the size of the remaining source, this will only copy all remaining elements in source.
    pub fn take(&mut self, n: usize) {
        let n = std::cmp::min(n, self.source_slice().len());
        if self.t1 != self.s0 {
            unsafe {
                let v = self.v.as_mut_ptr();
                std::ptr::copy(v.add(self.s0), v.add(self.t1), n);
            }
        }
        self.t1 += n;
        self.s0 += n;
    }

    /// Takes the next element from the source, if it exists
    pub fn pop_front(&mut self) -> Option<T> {
        if self.s0 < self.v.len() {
            self.s0 += 1;
            Some(unsafe { std::ptr::read(self.v.as_ptr().add(self.s0 - 1)) })
        } else {
            None
        }
    }

    fn drop_source(&mut self) {
        // use truncate to get rid of the source part, if any, calling drop as needed
        self.v.truncate(self.s0);
        // use set_len to get rid of the gap part between t1 and s0, not calling drop!
        unsafe {
            self.v.set_len(self.t1);
        }
        self.s0 = self.t1;
        // shorten the source part
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

impl<T> Drop for InPlaceVecBuilder<T> {
    fn drop(&mut self) {
        // drop the source part.
        self.drop_source();
        // explicitly drop the target part.
        self.v.clear();
    }
}

struct Builder<'a, T>(&'a RefCell<InPlaceVecBuilder<T>>);

impl<'a, T> Extend<T> for Builder<'a, T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let (min, _) = iter.size_hint();
        for x in iter {
            self.push(x, min)
        }
    }
}

impl<'a, T> Builder<'a, T> {
    pub fn push(&mut self, value: T, gap: usize) {
        self.0.borrow_mut().push(value, gap)
    }

    /// Take at most `n` elements from `iter` to the target. This will make room for `gap` elements if there is no space
    pub fn extend_from_iter<I: Iterator<Item = T>>(&mut self, iter: &mut I, n: usize, gap: usize) {
        if n > 0 {
            self.0.borrow_mut().ensure_capacity(n, gap);
            for _ in 0..n {
                if let Some(value) = iter.next() {
                    self.0.borrow_mut().push_unsafe(value);
                }
            }
        }
    }
}

struct BAI<T>(RefCell<InPlaceVecBuilder<T>>);

impl<T> BAI<T> {
    fn new(v: Vec<T>) -> Self {
        Self(RefCell::new(InPlaceVecBuilder::from(v)))
    }
    fn transform<'a, I, F>(&'a mut self, f: F)
    where
        F: Fn(Iter<'a, T>) -> I,
        I: Iterator<Item = T> + 'a,
        T: 'a,
    {
        let (mut b, i) = self.pair();
        b.extend(f(i));
    }
    fn pair<'a>(&'a mut self) -> (Builder<'a, T>, Iter<'a, T>) {
        (Builder(&self.0), Iter(&self.0))
    }
    fn into_vec(self) -> Vec<T> {
        self.0.into_inner().into_vec()
    }
}

struct Iter<'a, T>(&'a RefCell<InPlaceVecBuilder<T>>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.borrow_mut().pop_front()
    }
}

#[cfg(test)]
mod tests {
    extern crate testdrop;
    use super::*;
    use testdrop::{Item, TestDrop};

    fn everything_dropped<'a, F>(td: &'a TestDrop, n: usize, f: F)
    where
        F: Fn(Vec<Item<'a>>, Vec<Item<'a>>) -> InPlaceVecBuilder<Item<'a>>,
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
            let mut res: InPlaceVecBuilder<Item> = a.into();
            for x in b.into_iter() {
                res.push(x, 100);
            }
            res
        })
    }

    #[test]
    fn source_move_some() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: InPlaceVecBuilder<Item> = a.into();
            res.take(3);
            res
        })
    }

    #[test]
    fn source_move_all() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: InPlaceVecBuilder<Item> = a.into();
            res.take(10);
            res
        })
    }

    #[test]
    fn source_drop_some() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: InPlaceVecBuilder<Item> = a.into();
            res.skip(3);
            res
        })
    }

    #[test]
    fn source_drop_all() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: InPlaceVecBuilder<Item> = a.into();
            res.skip(10);
            res
        })
    }

    #[test]
    fn source_pop_some() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: InPlaceVecBuilder<Item> = a.into();
            res.pop_front();
            res.pop_front();
            res.pop_front();
            res
        })
    }

    #[test]
    fn builder() {
        let a = vec![1, 2, 3];
        let mut a = BAI::new(a);
        let (mut b, i) = a.pair();
        b.extend(i.map(|x| x * 2));
        let r: Vec<_> = a.into_vec();
        assert_eq!(r, vec![2, 4, 6]);
    }
}

pub fn demo() {
    let a = vec![1, 2, 3];
    let mut a = BAI::new(a);
    let (mut b, i) = a.pair();
    b.extend(i.map(|x| x * 2));
    let r: Vec<_> = a.into_vec();
    assert_eq!(r, vec![2, 4, 6]);
}
