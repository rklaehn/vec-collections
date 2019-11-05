// an iterator for a slice that allows random access read as well as dropping or taking multiple elements from the front
use std::cmp::Ordering::*;
use std::cmp::{max, min};
use std::iter::Peekable;
use std::marker::PhantomData;

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

pub trait SortedPairIterator<K, V>: Iterator<Item = (K, V)> {
    fn peek(&mut self) -> Option<&(K, V)>;
}

impl<K, V, I: Iterator<Item = (K, V)>> SortedPairIterator<K, V> for SortedPairIter<I> {
    fn peek(&mut self) -> Option<&I::Item> {
        self.0.peek()
    }
}

pub trait SortedIterator<K>: Iterator<Item = K> {
    fn peek(&mut self) -> Option<&K>;
}

pub struct SortedIter<I: Iterator>(Peekable<I>);

impl<K, I: Iterator<Item = K>> SortedIterator<K> for SortedIter<I> {
    fn peek(&mut self) -> Option<&I::Item> {
        self.0.peek()
    }
}

impl<K, I: Iterator<Item = K>> Iterator for SortedIter<I> {
    type Item = K;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K, I: Iterator<Item = K>> SortedIter<I> {
    fn new(iter: I) -> Self {
        Self(iter.peekable())
    }
}

struct Intersection<K, I, J> {
    a: I,
    b: J,
    x: PhantomData<K>,
}

impl<K: Ord, I: SortedIterator<K>, J: SortedIterator<K>> Iterator for Intersection<K, I, J> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        while let (Some(a), Some(b)) = (self.a.peek(), self.b.peek()) {
            match a.cmp(&b) {
                Less => {
                    self.a.next();
                }
                Greater => {
                    self.b.next();
                }
                Equal => {
                    self.b.next();
                    return self.a.next();
                }
            }
        }
        None
    }
}

struct Union<K, I, J> {
    a: I,
    b: J,
    x: PhantomData<K>,
}

impl<K: Ord, I: SortedIterator<K>, J: SortedIterator<K>> Iterator for Union<K, I, J> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        if let (Some(ak), Some(bk)) = (self.a.peek(), self.b.peek()) {
            match ak.cmp(&bk) {
                Less => self.a.next(),
                Greater => self.b.next(),
                Equal => {
                    self.b.next();
                    self.a.next()
                }
            }
        } else {
            self.a.next().or_else(|| self.b.next())
        }
    }
}

impl<K: Ord, I: Iterator<Item = K>> SortedIter<I> {
    pub fn take(self, n: usize) -> impl SortedIterator<K> {
        SortedIter::new(self.0.take(n))
    }
    pub fn take_while<P: FnMut(&I::Item) -> bool>(self, predicate: P) -> impl SortedIterator<K> {
        SortedIter::new(self.0.take_while(predicate))
    }
    pub fn skip(self, n: usize) -> impl SortedIterator<K> {
        SortedIter::new(self.0.skip(n))
    }
    pub fn skip_while<P: FnMut(&I::Item) -> bool>(self, predicate: P) -> impl SortedIterator<K> {
        SortedIter::new(self.0.skip_while(predicate))
    }
    pub fn filter<P: FnMut(&I::Item) -> bool>(self, predicate: P) -> impl SortedIterator<K> {
        SortedIter::new(self.0.filter(predicate))
    }
    pub fn step_by(self, step: usize) -> impl SortedIterator<K> {
        SortedIter::new(self.0.step_by(step))
    }
    pub fn intersection<J: SortedIterator<K>>(self, that: J) -> impl SortedIterator<K> {
        SortedIter::new(Intersection {
            a: self,
            b: that,
            x: PhantomData,
        })
    }
    pub fn union<J: SortedIterator<K>>(self, that: J) -> impl SortedIterator<K> {
        SortedIter::new(Union {
            a: self,
            b: that,
            x: PhantomData,
        })
    }
}

pub struct SortedPairIter<I: Iterator>(Peekable<I>);

impl<K, V, I: Iterator<Item = (K, V)>> SortedPairIter<I> {
    fn new(iter: I) -> Self {
        Self(iter.peekable())
    }
}

struct InnerJoin<K, A, B, R, I, J, F> {
    a: I,
    b: J,
    f: F,
    x: PhantomData<(K, A, B, R)>,
}

struct LeftJoin<K, A, B, R, I, J, F> {
    a: I,
    b: J,
    f: F,
    x: PhantomData<(K, A, B, R)>,
}

struct RightJoin<K, A, B, R, I, J, F> {
    a: I,
    b: J,
    f: F,
    x: PhantomData<(K, A, B, R)>,
}

struct OuterJoin<K, A, B, R, I, J, F> {
    a: I,
    b: J,
    f: F,
    x: PhantomData<(K, A, B, R)>,
}

impl<K, A, B, R, I, J, F> Iterator for InnerJoin<K, A, B, R, I, J, F>
where
    K: Ord,
    I: SortedPairIterator<K, A>,
    J: SortedPairIterator<K, B>,
    F: FnMut(A, B) -> R,
{
    type Item = (K, R);

    fn next(&mut self) -> Option<Self::Item> {
        while let (Some((ak, _)), Some((bk, _))) = (self.a.peek(), self.b.peek()) {
            match ak.cmp(&bk) {
                Less => {
                    self.a.next();
                }
                Greater => {
                    self.b.next();
                }
                Equal => {
                    if let (Some((ak, av)), Some((_, bv))) = (self.a.next(), self.b.next()) {
                        let r = (self.f)(av, bv);
                        return Some((ak, r));
                    } else {
                        unreachable!();
                    }
                }
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (amin, amax) = self.a.size_hint();
        let (bmin, bmax) = self.b.size_hint();
        let rmin = min(amin, bmin);
        let rmax = amax.and_then(|amax| bmax.map(|bmax| min(amax, bmax)));
        (rmin, rmax)
    }
}

impl<K, A, B, R, I, J, F> Iterator for LeftJoin<K, A, B, R, I, J, F>
where
    K: Ord,
    I: SortedPairIterator<K, A>,
    J: SortedPairIterator<K, B>,
    F: FnMut(A, Option<B>) -> R,
{
    type Item = (K, R);

    fn next(&mut self) -> Option<Self::Item> {
        let (ak, _) = self.a.peek()?;
        while let Some((bk, _)) = self.b.peek() {
            if bk < ak {
                self.b.next();
            } else {
                break;
            }
        }
        let (ak, av) = self.a.next()?;
        let r = (self.f)(av, self.b.next().map(|(_, bv)| bv));
        Some((ak, r))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.a.size_hint()
    }
}

impl<K, A, B, R, I, J, F> Iterator for RightJoin<K, A, B, R, I, J, F>
where
    K: Ord,
    I: SortedPairIterator<K, A>,
    J: SortedPairIterator<K, B>,
    F: FnMut(Option<A>, B) -> R,
{
    type Item = (K, R);

    fn next(&mut self) -> Option<Self::Item> {
        let (bk, _) = self.b.peek()?;
        while let Some((ak, _)) = self.a.peek() {
            if ak < bk {
                self.a.next();
            } else {
                break;
            }
        }
        let (bk, bv) = self.b.next()?;
        let r = (self.f)(self.a.next().map(|(_, av)| av), bv);
        Some((bk, r))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.b.size_hint()
    }
}

impl<K, A, B, R, I, J, F> OuterJoin<K, A, B, R, I, J, F>
where
    K: Ord,
    I: SortedPairIterator<K, A>,
    J: SortedPairIterator<K, B>,
    F: FnMut(Option<A>, Option<B>) -> R,
{
    fn from_a(&mut self) -> Option<(K, R)> {
        self.a.next().map(|(ak, av)| (ak, (self.f)(Some(av), None)))
    }

    fn from_b(&mut self) -> Option<(K, R)> {
        self.b.next().map(|(bk, bv)| (bk, (self.f)(None, Some(bv))))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (amin, amax) = self.a.size_hint();
        let (bmin, bmax) = self.b.size_hint();
        let rmin = max(amin, bmin);
        let rmax = amax.and_then(|amax| bmax.map(|bmax| max(amax, bmax)));
        (rmin, rmax)
    }
}

impl<K, A, B, R, I, J, F> Iterator for OuterJoin<K, A, B, R, I, J, F>
where
    K: Ord,
    I: SortedPairIterator<K, A>,
    J: SortedPairIterator<K, B>,
    F: FnMut(Option<A>, Option<B>) -> R,
{
    type Item = (K, R);

    fn next(&mut self) -> Option<Self::Item> {
        if let (Some((ak, _)), Some((bk, _))) = (self.a.peek(), self.b.peek()) {
            match ak.cmp(&bk) {
                Less => self.from_a(),
                Greater => self.from_b(),
                Equal => self.a.next().and_then(|(ak, av)| {
                    self.b
                        .next()
                        .map(|(_, bv)| (ak, (self.f)(Some(av), Some(bv))))
                }),
            }
        } else {
            self.from_a().or_else(|| self.from_b())
        }
    }
}

impl<K: Ord, V, I: Iterator<Item = (K, V)>> SortedPairIter<I> {
    pub fn take(self, n: usize) -> impl SortedPairIterator<K, V> {
        SortedPairIter::new(self.0.take(n))
    }
    pub fn take_while<P: FnMut(&I::Item) -> bool>(
        self,
        predicate: P,
    ) -> impl SortedPairIterator<K, V> {
        SortedPairIter::new(self.0.take_while(predicate))
    }
    pub fn skip(self, n: usize) -> impl SortedPairIterator<K, V> {
        SortedPairIter::new(self.0.skip(n))
    }
    pub fn skip_while<P: FnMut(&I::Item) -> bool>(
        self,
        predicate: P,
    ) -> impl SortedPairIterator<K, V> {
        SortedPairIter::new(self.0.skip_while(predicate))
    }
    pub fn filter<P: FnMut(&I::Item) -> bool>(self, predicate: P) -> impl SortedPairIterator<K, V> {
        SortedPairIter::new(self.0.filter(predicate))
    }
    pub fn step_by(self, step: usize) -> impl SortedPairIterator<K, V> {
        SortedPairIter::new(self.0.step_by(step))
    }
    pub fn map_values<W, F: (FnMut(V) -> W)>(self, mut f: F) -> impl SortedPairIterator<K, W> {
        SortedPairIter::new(self.0.map(move |(k, v)| (k, f(v))))
    }
    pub fn filter_map_values<W, F: (FnMut(V) -> Option<W>)>(
        self,
        mut f: F,
    ) -> impl SortedPairIterator<K, W> {
        SortedPairIter::new(self.0.filter_map(move |(k, v)| f(v).map(|w| (k, w))))
    }
    pub fn inner_join<W, R, J: SortedPairIterator<K, W>, F: FnMut(V, W) -> R>(
        self,
        rhs: J,
        f: F,
    ) -> impl SortedPairIterator<K, R> {
        SortedPairIter::new(InnerJoin {
            a: self,
            b: rhs,
            f,
            x: PhantomData,
        })
    }
    pub fn left_join<W, R, J: SortedPairIterator<K, W>, F: FnMut(V, Option<W>) -> R>(
        self,
        rhs: J,
        f: F,
    ) -> impl SortedPairIterator<K, R> {
        SortedPairIter::new(LeftJoin {
            a: self,
            b: rhs,
            f,
            x: PhantomData,
        })
    }
    pub fn right_join<W, R, J: SortedPairIterator<K, W>, F: FnMut(Option<V>, W) -> R>(
        self,
        rhs: J,
        f: F,
    ) -> impl SortedPairIterator<K, R> {
        SortedPairIter::new(RightJoin {
            a: self,
            b: rhs,
            f,
            x: PhantomData,
        })
    }
    pub fn outer_join<W, R, J: SortedPairIterator<K, W>, F: FnMut(Option<V>, Option<W>) -> R>(
        self,
        rhs: J,
        f: F,
    ) -> impl SortedPairIterator<K, R> {
        SortedPairIter::new(OuterJoin {
            a: self,
            b: rhs,
            f,
            x: PhantomData,
        })
    }
}

impl<K, V, I: Iterator<Item = (K, V)>> Iterator for SortedPairIter<I> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub fn test_join<'a>(a: Vec<(i32, i32)>, b: Vec<(i32, i32)>) -> Vec<(i32, i32)> {
    let a = SortedPairIter::new(a.into_iter());
    let b = SortedPairIter::new(b.into_iter());
    a.inner_join(b, |a, b| a + b).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorted_pair_iter() {
        let a = SortedPairIter::new((0..10).step_by(2).map(|k| (k, k)));
        let b = SortedPairIter::new((0..5).map(|k| (k, k)));
        // let z = b.take_while(|x| x.0 < 10);
        // let w = a.take(10).take(5);
        let r: Vec<_> = a.outer_join(b, |a, b| (a, b)).collect();
        println!("{:?}", r);
    }

    #[test]
    fn test_sorted_iter() {
        let a = SortedIter::new((1..20).step_by(2));
        let b = SortedIter::new(0..15);
        // let z = b.take_while(|x| x.0 < 10);
        // let w = a.take(10).take(5);
        let r: Vec<_> = a.union(b).collect();
        println!("{:?}", r);
    }
}
