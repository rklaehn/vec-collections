use crate::binary_merge::MergeOperation1;
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

struct OuterJoinOp<F>(F);
struct LeftJoinOp<F>(F);
struct RightJoinOp<F>(F);
struct InnerJoinOp<F>(F);

struct VecMergeState<'a, A, B, R> {
    a: SliceIterator<'a, A>,
    b: SliceIterator<'a, B>,
    r: Vec<R>,
}

impl<'a, A, B, R> VecMergeState<'a, A, B, R> {
    fn take_a(&mut self, n: usize) -> EarlyOut {
        self.a.drop(n);
        Some(())
    }

    fn take_b(&mut self, n: usize) -> EarlyOut {
        self.b.drop(n);
        Some(())
    }

    pub fn into_vec(self) -> Vec<R> {
        self.r
    }

    pub fn new(a: &'a [A], b: &'a [B], r: Vec<R>) -> Self {
        Self {
            a: SliceIterator::new(a),
            b: SliceIterator::new(b),
            r,
        }
    }

    pub fn merge<O: MergeOperation1<'a, A, B, Self>>(
        a: &'a [A],
        b: &'a [B],
        o: O,
    ) -> Vec<R> {
        let t: Vec<R> = Vec::new();
        let mut state = VecMergeState::new(a, b, t);
        o.merge(&mut state);
        state.into_vec()
    }
}

impl<'a, A, B, R> MergeStateRead<A, B> for VecMergeState<'a, A, B, R> {
    fn a_slice(&self) -> &[A] {
        self.a.as_slice()
    }
    fn b_slice(&self) -> &[B] {
        self.b.as_slice()
    }
}

type PairMergeState<'a, K, A, B, R> = VecMergeState<'a, (K, A), (K, B), (K, R)>;

impl<'a, K: Ord + Clone, A, B, R, F: Fn(OuterJoinArg<&A, &B>) -> R>
    MergeOperation1<'a, (K, A), (K, B), PairMergeState<'a, K, A, B, R>> for OuterJoinOp<F>
{
    fn cmp(&self, a: &(K, A), b: &(K, B)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        for _ in 0..n {
            if let Some((k, a)) = m.a.next() {
                let arg = OuterJoinArg::Left(a);
                let res = (self.0)(arg);
                m.r.push((k.clone(), res));
            }
        }
    }
    fn from_b(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        for _ in 0..n {
            if let Some((k, b)) = m.b.next() {
                let arg = OuterJoinArg::Right(b);
                let res = (self.0)(arg);
                m.r.push((k.clone(), res));
            }
        }
    }
    fn collision(&self, m: &mut PairMergeState<'a, K, A, B, R>) {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                let arg = OuterJoinArg::Both(a, b);
                let res = (self.0)(arg);
                m.r.push((k.clone(), res));
            }
        }
    }
}

impl<'a, K: Ord + Clone, A, B, R, F: Fn(LeftJoinArg<&A, &B>) -> R>
    MergeOperation1<'a, (K, A), (K, B), PairMergeState<'a, K, A, B, R>> for LeftJoinOp<F>
{
    fn cmp(&self, a: &(K, A), b: &(K, B)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        for _ in 0..n {
            if let Some((k, a)) = m.a.next() {
                let arg = LeftJoinArg::Left(a);
                let res = (self.0)(arg);
                m.r.push((k.clone(), res));
            }
        }
    }
    fn from_b(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        m.b.drop(n);
    }
    fn collision(&self, m: &mut PairMergeState<'a, K, A, B, R>) {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                let arg = LeftJoinArg::Both(a, b);
                let res = (self.0)(arg);
                m.r.push((k.clone(), res));
            }
        }
    }
}

impl<'a, K: Ord + Clone, A, B, R, F: Fn(RightJoinArg<&A, &B>) -> R>
    MergeOperation1<'a, (K, A), (K, B), PairMergeState<'a, K, A, B, R>> for RightJoinOp<F>
{
    fn cmp(&self, a: &(K, A), b: &(K, B)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        m.a.drop(n);
    }
    fn from_b(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        for _ in 0..n {
            if let Some((k, b)) = m.b.next() {
                let arg = RightJoinArg::Right(b);
                let res = (self.0)(arg);
                m.r.push((k.clone(), res));
            }
        }
    }
    fn collision(&self, m: &mut PairMergeState<'a, K, A, B, R>) {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                let arg = RightJoinArg::Both(a, b);
                let res = (self.0)(arg);
                m.r.push((k.clone(), res));
            }
        }
    }
}

impl<'a, K: Ord + Clone, A, B, R, F: Fn(&A, &B) -> R>
    MergeOperation1<'a, (K, A), (K, B), PairMergeState<'a, K, A, B, R>> for InnerJoinOp<F>
{
    fn cmp(&self, a: &(K, A), b: &(K, B)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        m.a.drop(n);
    }
    fn from_b(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        m.b.drop(n);
    }
    fn collision(&self, m: &mut PairMergeState<'a, K, A, B, R>) {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                let res = (self.0)(a, b);
                m.r.push((k.clone(), res));
            }
        }
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

    pub fn outer_join<W: Clone, R, F: Fn(OuterJoinArg<&V, &W>) -> R>(
        &self,
        that: &ArrayMap<K, W>,
        f: F,
    ) -> ArrayMap<K, R> {
        ArrayMap::<K, R>::from_sorted_vec(VecMergeState::merge(
            self.0.as_slice(),
            that.0.as_slice(),
            OuterJoinOp(f),
        ))
    }

    pub fn left_join<W: Clone, R, F: Fn(LeftJoinArg<&V, &W>) -> R>(
        &self,
        that: &ArrayMap<K, W>,
        f: F,
    ) -> ArrayMap<K, R> {
        ArrayMap::<K, R>::from_sorted_vec(VecMergeState::merge(
            self.0.as_slice(),
            that.0.as_slice(),
            LeftJoinOp(f),
        ))
    }

    pub fn right_join<W: Clone, R, F: Fn(RightJoinArg<&V, &W>) -> R>(
        &self,
        that: &ArrayMap<K, W>,
        f: F,
    ) -> ArrayMap<K, R> {
        ArrayMap::<K, R>::from_sorted_vec(VecMergeState::merge(
            self.0.as_slice(),
            that.0.as_slice(),
            RightJoinOp(f),
        ))
    }

    pub fn inner_join<W: Clone, R, F: Fn(&V, &W) -> R>(
        &self,
        that: &ArrayMap<K, W>,
        f: F,
    ) -> ArrayMap<K, R> {
        ArrayMap::<K, R>::from_sorted_vec(VecMergeState::merge(
            self.0.as_slice(),
            that.0.as_slice(),
            InnerJoinOp(f),
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
    use std::collections::BTreeMap;
    use OuterJoinArg::*;

    type Test = ArrayMap<i32, i32>;
    type Ref = BTreeMap<i32, i32>;

    fn outer_join_reference(a: &Ref, b: &Ref) -> Ref {
        let mut r = a.clone();
        for (k, v) in b.clone().into_iter() {
            r.insert(k, v);
        }
        r
    }

    fn inner_join_reference(a: &Ref, b: &Ref) -> Ref {
        let mut r: Ref = BTreeMap::new();
        for (k, v) in a.clone().into_iter() {
            if b.contains_key(&k) {
                r.insert(k, v);
            }
        }
        r
    }

    quickcheck! {
        fn outer_join(a: Ref, b: Ref) -> bool {
            let expected: Test = outer_join_reference(&a, &b).into();
            let a: Test = a.into();
            let b: Test = b.into();
            let actual = a.outer_join(&b, |arg| match arg {
                Left(a) => a.clone(),
                Right(b) => b.clone(),
                Both(_, b) => b.clone(),
            });
            expected == actual
        }

        fn inner_join(a: Ref, b: Ref) -> bool {
            let expected: Test = inner_join_reference(&a, &b).into();
            let a: Test = a.into();
            let b: Test = b.into();
            let actual = a.inner_join(&b, |a,b| a.clone());
            expected == actual
        }
    }

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
        let r = outer_join_reference(&a, &b);
        let a: Test = a.into();
        let b: Test = b.into();
        let expected: Test = r.into();
        let actual = a.outer_join(&b, |arg| match arg {
            Left(a) => a.clone(),
            Right(b) => b.clone(),
            Both(_, b) => b.clone(),
        });
        assert_eq!(actual, expected);
        println!("{:?}", actual);
    }
}
