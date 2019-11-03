use crate::binary_merge::MergeStateRead;
use crate::{EarlyOut, MergeOperation, MergeState};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::Debug;

#[derive(Hash, Clone, Eq, PartialEq)]
struct ArrayMap<K, V>(Vec<(K, V)>);

impl<K: Debug, V: Debug> Debug for ArrayMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}

struct MapLeftUnionOp();

impl<'a, K: Ord, V, I: MergeState<(K, V), (K, V)>> MergeOperation<'a, (K, V), (K, V), I>
    for MapLeftUnionOp
{
    fn cmp(&self, a: &(K, V), b: &(K, V)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_a(n)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_b(n)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.move_a(1)?;
        m.skip_b(1)
    }
}

struct SliceIterator<'a, T>(&'a [T]);

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
    fn new(slice: &'a [T]) -> Self {
        Self(slice)
    }

    fn as_slice(&self) -> &[T] {
        self.0
    }

    fn drop(&mut self, n: usize) {
        self.0 = &self.0[n..];
    }
}

enum LeftJoinArg<A, B> {
    Left(A),
    Both(A, B),
}

enum RightJoinArg<A, B> {
    Both(A, B),
    Right(B),
}

enum OuterJoinArg<A, B> {
    Left(A),
    Right(B),
    Both(A, B),
}

struct MapOuterJoinOp<A, B, R, F: Fn(OuterJoinArg<A, B>) -> R> {
    f: F,
    x: std::marker::PhantomData<(A, B, R)>,
}

impl<A, B, R, F: Fn(OuterJoinArg<A, B>) -> R> MapOuterJoinOp<A, B, R, F> {
    fn new(f: F) -> Self {
        Self {
            f,
            x: std::marker::PhantomData,
        }
    }
}

struct VecMergeState<'a, K, A, B, R> {
    a: SliceIterator<'a, (K, A)>,
    b: SliceIterator<'a, (K, B)>,
    r: Vec<(K, R)>,
}

impl<'a, K, A, B, R> VecMergeState<'a, K, A, B, R> {
    fn take_a(&mut self, n: usize) -> EarlyOut {
        self.a.drop(n);
        Some(())
    }

    fn take_b(&mut self, n: usize) -> EarlyOut {
        self.b.drop(n);
        Some(())
    }

    pub fn into_vec(self) -> Vec<(K, R)> {
        self.r
    }

    pub fn new(a: &'a [(K, A)], b: &'a [(K, B)], r: Vec<(K, R)>) -> Self {
        Self {
            a: SliceIterator::new(a),
            b: SliceIterator::new(b),
            r,
        }
    }

    pub fn merge<O: MergeOperation<'a, (K, A), (K, B), Self>>(
        a: &'a [(K, A)],
        b: &'a [(K, B)],
        o: O,
    ) -> Vec<(K, R)> {
        let t: Vec<(K, R)> = Vec::new();
        let mut state = VecMergeState::new(a, b, t);
        o.merge(&mut state);
        state.into_vec()
    }
}

impl<'a, K, A, B, R> MergeStateRead<(K, A), (K, B)> for VecMergeState<'a, K, A, B, R> {
    fn a_slice(&self) -> &[(K, A)] {
        self.a.as_slice()
    }
    fn b_slice(&self) -> &[(K, B)] {
        self.b.as_slice()
    }
}

impl<'a, K: Ord + Clone, A: Clone, B: Clone, R, F: Fn(OuterJoinArg<A, B>) -> R>
    MergeOperation<'a, (K, A), (K, B), VecMergeState<'a, K, A, B, R>>
    for MapOuterJoinOp<A, B, R, F>
{
    fn cmp(&self, a: &(K, A), b: &(K, B)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut VecMergeState<'a, K, A, B, R>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, a)) = m.a.next() {
                let arg: OuterJoinArg<A, B> = OuterJoinArg::Left(a.clone());
                let res = (self.f)(arg);
                m.r.push((k.clone(), res));
            }
        }
        Some(())
    }
    fn from_b(&self, m: &mut VecMergeState<'a, K, A, B, R>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, b)) = m.b.next() {
                let arg: OuterJoinArg<A, B> = OuterJoinArg::Right(b.clone());
                let res = (self.f)(arg);
                m.r.push((k.clone(), res));
            }
        }
        Some(())
    }
    fn collision(&self, m: &mut VecMergeState<'a, K, A, B, R>) -> EarlyOut {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                let arg: OuterJoinArg<A, B> = OuterJoinArg::Both(a.clone(), b.clone());
                let res = (self.f)(arg);
                m.r.push((k.clone(), res));
            }
        }
        Some(())
    }
}

impl<K, V> ArrayMap<K, V> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn retain<F: FnMut((&K, &V)) -> bool>(&mut self, mut f: F) {
        self.0.retain(|entry| f((&entry.0, &entry.1)))
    }

    pub fn map_values<R, F: FnMut(V) -> R>(self, mut f: F) -> ArrayMap<K, R> {
        ArrayMap::from_sorted_vec(
            self.0
                .into_iter()
                .map(|entry| (entry.0, f(entry.1)))
                .collect(),
        )
    }

    fn from_sorted_vec(v: Vec<(K, V)>) -> Self {
        Self(v)
    }
}

impl<K, V> From<std::collections::BTreeMap<K, V>> for ArrayMap<K, V> {
    fn from(value: std::collections::BTreeMap<K, V>) -> Self {
        let elements: Vec<(K, V)> = value.into_iter().collect();
        Self::from_sorted_vec(elements)
    }
}

impl<K: Ord + Clone, V: Clone> ArrayMap<K, V> {
    pub fn single(k: K, v: V) -> Self {
        Self::from_sorted_vec(vec![(k, v)])
    }

    pub fn outer_join<W: Clone, R, F: Fn(OuterJoinArg<V, W>) -> R>(
        &self,
        that: &ArrayMap<K, W>,
        f: F,
    ) -> ArrayMap<K, R> {
        ArrayMap::<K, R>::from_sorted_vec(VecMergeState::merge(
            self.0.as_slice(),
            that.0.as_slice(),
            MapOuterJoinOp::new(f),
        ))
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let elements = self.0.as_slice();
        match elements.binary_search_by(|p| p.0.borrow().cmp(key)) {
            Ok(index) => Some(&elements[index].1),
            Err(_) => None,
        }
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let elements = self.0.as_mut_slice();
        match elements.binary_search_by(|p| p.0.borrow().cmp(key)) {
            Ok(index) => Some(&mut elements[index].1),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::btreemap;
    use OuterJoinArg::*;

    #[test]
    fn smoke_test() {
        let a = btreemap! {
            1 => 1,
            2 => 3,
        };
        let b = btreemap! {
            1 => 2,
            3 => 4,
        };
        let mut r = a.clone();
        for (k, v) in b.clone().into_iter() {
            r.insert(k, v);
        }
        let a: ArrayMap<i32, i32> = a.into();
        let b: ArrayMap<i32, i32> = b.into();
        let expected: ArrayMap<i32, i32> = r.into();
        let actual = a.outer_join(&b, |arg| match arg {
            Left(a) => a,
            Right(b) => b,
            Both(_, b) => b,
        });
        assert_eq!(actual, expected);
        println!("{:?}", actual);
    }
}
