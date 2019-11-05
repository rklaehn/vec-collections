//! A data structure for in place modification of vecs
#![deny(warnings)]
#![deny(missing_docs)]
use std::fmt::Debug;
use std::mem::MaybeUninit;

/// A contiguous chunk of memory that is logically divided into a source and a target part
pub struct FlipBuffer<T> {
    v: Vec<T>,
    t1: usize,
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

impl<T> From<Vec<T>> for FlipBuffer<T> {
    fn from(value: Vec<T>) -> Self {
        Self::source_from_vec(value)
    }
}

impl<T> Into<Vec<T>> for FlipBuffer<T> {
    fn into(self) -> Vec<T> {
        self.target_into_vec()
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
    /// ensure that we have at least `n` space. If we have less than n, we will make `capacity` space
    fn ensure_capacity(&mut self, n: usize, capacity: usize) {
        // ensure we have space!
        if self.t1 + n > self.s0 {
            let capacity = std::cmp::max(capacity, n);
            // insert missing uninitialized dummy elements before s0
            self.v
                .splice(self.s0..self.s0, Spacer::<T>::sized(capacity));
            // move s0
            self.s0 += capacity;
        }
    }
    /// Take at most n elements from `iter` to the target. This will make room for `capacity` elements if there is no space
    pub fn target_extend_from_iter<I: Iterator<Item = T>>(
        &mut self,
        iter: &mut I,
        n: usize,
        capacity: usize,
    ) {
        if n > 0 {
            self.ensure_capacity(n, capacity);
            for _ in 0..n {
                if let Some(value) = iter.next() {
                    self.set(self.t1, value);
                    self.t1 += 1;
                }
            }
        }
    }
    /// Push a value to the target. This will make room for `capacity` elements if there is no space
    pub fn target_push(&mut self, value: T, capacity: usize) {
        // ensure we have space!
        self.ensure_capacity(1, capacity);
        self.set(self.t1, value);
        self.t1 += 1;
    }
    /// drop up to `n` elements from source
    pub fn source_drop(&mut self, n: usize) {
        let n = std::cmp::min(n, self.source_slice().len());
        for i in 0..n {
            // this is not a noop but is necessary to call drop!
            let _ = self.get(self.s0 + i);
        }
        self.s0 += n;
    }
    /// move up to n elements from source to target.
    /// If n is larger than the size of the remaining source, this will only copy all remaining elements in source.
    pub fn source_move(&mut self, n: usize) {
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
    pub fn source_pop_front(&mut self) -> Option<T> {
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
    fn drop_source_part(&mut self) {
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
            let _ = self.get(i);
        }
    }
    /// initializes the source part of the flip buffer with the given data
    fn source_from_vec(v: Vec<T>) -> Self {
        FlipBuffer { v, s0: 0, t1: 0 }
    }
    /// takes the target part of the flip buffer as a vec and drops the remaining source part, if any
    fn target_into_vec(self) -> Vec<T> {
        let mut r = self;
        r.drop_source_part();
        let mut t: Vec<T> = Vec::new();
        std::mem::swap(&mut t, &mut r.v);
        t
    }
}

// impl<T: Clone> FlipBuffer<T> {
//     /// extend from the given slice
//     pub fn target_extend_from_slice(&mut self, slice: &[T], capacity: usize) {
//         let needed = slice.len();
//         // ensure we have space!
//         if self.t1 + needed < self.s0 {
//             let capacity = std::cmp::max(capacity, needed);
//             // insert missing uninitialized dummy elements before s0
//             self.v
//                 .splice(self.s0..self.s0, Spacer::<T>::sized(capacity));
//             // move s0
//             self.s0 += capacity;
//         }
//         for elem in slice {
//             self.set(self.t1, elem.clone());
//             self.t1 += 1;
//         }
//     }
// }

impl<T> Drop for FlipBuffer<T> {
    fn drop(&mut self) {
        self.drop_source_part();
    }
}

struct Spacer<T>(std::marker::PhantomData<T>);

/// This thing is evil incarnate. It allows you to create single or multiple T from uninitialized memory
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
            res.source_move(3);
            res
        })
    }

    #[test]
    fn source_move_all() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: FlipBuffer<Item> = a.into();
            res.source_move(10);
            res
        })
    }

    #[test]
    fn source_drop_some() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: FlipBuffer<Item> = a.into();
            res.source_drop(3);
            res
        })
    }

    #[test]
    fn source_drop_all() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: FlipBuffer<Item> = a.into();
            res.source_drop(10);
            res
        })
    }

    #[test]
    fn source_pop_some() {
        everything_dropped(&TestDrop::new(), 10, |a, _| {
            let mut res: FlipBuffer<Item> = a.into();
            res.source_pop_front();
            res.source_pop_front();
            res.source_pop_front();
            res
        })
    }
}
