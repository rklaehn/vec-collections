// an iterator for a slice that allows random access read as well as dropping or taking multiple elements from the front
use core::iter::Take;
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::fmt::Debug;
use std::iter::Filter;
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

trait SortedPairIterator<K, V>: Iterator<Item = (K, V)> {
    fn peek(&mut self) -> Option<&(K, V)>;
    // fn next() -> Option<(K, V)>;
    // fn size_hint() -> (usize, Option<usize>);
}

impl<K, V, I: Iterator<Item = (K, V)>> SortedPairIterator<K, V> for SPI<Peekable<I>> {
    fn peek(&mut self) -> Option<&I::Item> {
        self.inner.peek()
    }
}

pub(crate) struct SPI<I> {
    inner: I,
}

impl<K, V, I: Iterator<Item = (K, V)>> SPI<Peekable<I>> {
    fn new(iter: I) -> Self {
        Self {
            inner: iter.peekable(),
        }
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

impl<K: Ord + Debug, V: Debug, I: Iterator<Item = (K, V)>> SPI<Peekable<I>> {
    fn take(self, n: usize) -> impl SortedPairIterator<K, V> {
        SPI::new(self.inner.take(n))
    }
    fn filter<P: FnMut(&I::Item) -> bool>(self, predicate: P) -> impl SortedPairIterator<K, V> {
        SPI::new(self.inner.filter(predicate))
    }
    fn map_values<W, F: (FnMut(V) -> W)>(self, mut f: F) -> impl SortedPairIterator<K, W> {
        SPI::new(self.inner.map(move |(k, v)| (k, f(v))))
    }
    fn inner_join<W, R, J: SortedPairIterator<K, W>, F: FnMut(V, W) -> R>(
        self,
        rhs: J,
        f: F,
    ) -> impl SortedPairIterator<K, R> {
        SPI::new(InnerJoin {
            a: self,
            b: rhs,
            f,
            x: PhantomData,
        })
    }
    fn left_join<W, R, J: SortedPairIterator<K, W>, F: FnMut(V, Option<W>) -> R>(
        self,
        rhs: J,
        f: F,
    ) -> impl SortedPairIterator<K, R> {
        SPI::new(LeftJoin {
            a: self,
            b: rhs,
            f,
            x: PhantomData,
        })
    }
    fn right_join<W, R, J: SortedPairIterator<K, W>, F: FnMut(Option<V>, W) -> R>(
        self,
        rhs: J,
        f: F,
    ) -> impl SortedPairIterator<K, R> {
        SPI::new(RightJoin {
            a: self,
            b: rhs,
            f,
            x: PhantomData,
        })
    }
    fn outer_join<W, R, J: SortedPairIterator<K, W>, F: FnMut(Option<V>, Option<W>) -> R>(
        self,
        rhs: J,
        f: F,
    ) -> impl SortedPairIterator<K, R> {
        SPI::new(OuterJoin {
            a: self,
            b: rhs,
            f,
            x: PhantomData,
        })
    }
}

impl<K, V, I: Iterator<Item = (K, V)>> Iterator for SPI<I> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorted_iter() {
        let a = SPI::new((0..10).step_by(2).map(|k| (k, k)));
        let b = SPI::new((0..5).map(|k| (k, k)));
        // let z = b.take_while(|x| x.0 < 10);
        // let w = a.take(10).take(5);
        let r: Vec<_> = a.outer_join(b, |a, b| (a, b)).collect();
        println!("{:?}", r);
    }
}
