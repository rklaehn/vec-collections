use std::mem::MaybeUninit;

pub struct UnsafeFlipBuffer<T> {
    v: Vec<T>,
    t1: usize,
    s0: usize,
}

impl<T> From<Vec<T>> for UnsafeFlipBuffer<T> {
    fn from(value: Vec<T>) -> Self {
        Self::source_from_vec(value)
    }
}

impl<T> Into<Vec<T>> for UnsafeFlipBuffer<T> {
    fn into(self) -> Vec<T> {
        self.target_into_vec()
    }
}

impl<T> UnsafeFlipBuffer<T> {
    pub fn target_slice(&self) -> &[T] {
        &self.v[..self.t1]
    }
    pub fn source_slice(&self) -> &[T] {
        &self.v[self.s0..]
    }
    pub fn target_push(&mut self, value: T, missing: usize) {
        // ensure we have space!
        if self.t1 == self.s0 {
            debug_assert!(missing >= 1);
            // insert missing uninitialized dummy elements before s0
            self.v.splice(self.s0..self.s0, Spacer::<T>::sized(missing));
            // move s0
            self.s0 += missing;
        }
        self.v[self.t1] = value;
        self.t1 += 1;
    }
    // drop up to n elements from source
    pub fn drop_front(&mut self, n: usize) {
        let n = std::cmp::min(n, self.source_slice().len());
        for i in 0..n {
            // this is not a noop but is necessary to call drop!
            let _ = self.at(self.s0 + i);
        }
        self.s0 += n;
    }
    /// move up to n elements from source to target.
    /// If n is larger than the size of the remaining source, this will only copy all remaining elements in source.
    pub fn move_front(&mut self, n: usize) {
        let n = std::cmp::min(n, self.source_slice().len());
        if self.t1 != self.s0 {
            for i in 0..n {
                self.v[self.t1 + i] = self.at(self.s0 + i);
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
            Some(self.at(old))
        } else {
            None
        }
    }
    fn at(&self, offset: usize) -> T {
        unsafe { std::ptr::read(self.v.as_ptr().add(offset)) }
    }
    fn drop_source_part(&mut self) {
        // truncate so that just the target part remains
        // this will do nothing to the remaining part of the vector, no drop
        // this is done first so we don't double drop if there is a panic in a drop.
        unsafe {
            self.v.set_len(self.t1);
        }
        // drop all remaining elements of the source, if any
        // in the unlikely case of a panic in drop, we will just stop dropping and thus leak,
        // but not double-drop!
        while let Some(_) = self.source_pop_front() {}
    }
    /// initializes the source part of the flip buffer with the given data
    fn source_from_vec(v: Vec<T>) -> Self {
        UnsafeFlipBuffer { v, s0: 0, t1: 0 }
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

impl<T> Drop for UnsafeFlipBuffer<T> {
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