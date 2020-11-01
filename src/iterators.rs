/// An interator that is guaranteed to be sorted by item
pub struct VecSetIter<I> {
    i: I,
}

impl<I> sorted_iter::sorted_iterator::SortedByItem for VecSetIter<I> {}

impl<I: Iterator> VecSetIter<I> {
    pub(crate) fn new(i: I) -> Self {
        Self { i }
    }
}

impl<I: Iterator> Iterator for VecSetIter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.i.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.i.size_hint()
    }
}

/// An interator that is guaranteed to be sorted by key
pub struct VecMapIter<I> {
    i: I,
}

impl<I> sorted_iter::sorted_pair_iterator::SortedByKey for VecMapIter<I> {}

impl<I: Iterator> VecMapIter<I> {
    pub(crate) fn new(i: I) -> Self {
        Self { i }
    }
}

impl<I: Iterator> Iterator for VecMapIter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.i.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.i.size_hint()
    }
}

pub(crate) struct SliceIterator<'a, T>(pub &'a [T]);

impl<'a, T> Iterator for SliceIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            None
        } else {
            let res: Self::Item = &self.0[0];
            self.0 = &self.0[1..];
            Some(res)
        }
    }
}

impl<'a, T> SliceIterator<'a, T> {
    pub fn as_slice(&self) -> &[T] {
        self.0
    }

    pub(crate) fn drop_front(&mut self, n: usize) {
        self.0 = &self.0[n..];
    }

    pub(crate) fn take_front(&mut self, n: usize) -> &'a [T] {
        let res = &self.0[..n];
        self.0 = &self.0[n..];
        res
    }
}
