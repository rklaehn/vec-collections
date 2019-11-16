use crate::binary_merge::MergeOperation;
use crate::dedup::{sort_and_dedup_by_key, Keep};
use crate::iterators::SliceIterator;
use crate::merge_state::{MergeStateMut, UnsafeInPlaceMergeState, VecMergeState};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::iter::FromIterator;

#[derive(Hash, Clone, Eq, PartialEq)]
pub struct VecMap<K, V>(Vec<(K, V)>);

impl<K, V> Default for VecMap<K, V> {
    fn default() -> Self {
        Self(Vec::default())
    }
}

impl<K: Debug, V: Debug> Debug for VecMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}

struct CombineOp<F, K>(F, std::marker::PhantomData<K>);

impl<K: Ord, V, F: Fn(V, V) -> V>
    MergeOperation<(K, V), (K, V), UnsafeInPlaceMergeState<(K, V), (K, V)>> for CombineOp<F, K>
{
    fn cmp(&self, a: &(K, V), b: &(K, V)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut UnsafeInPlaceMergeState<(K, V), (K, V)>, n: usize) {
        m.move_a(n);
    }
    fn from_b(&self, m: &mut UnsafeInPlaceMergeState<(K, V), (K, V)>, n: usize) {
        m.move_b(n);
    }
    fn collision(&self, m: &mut UnsafeInPlaceMergeState<(K, V), (K, V)>) {
        if let (Some((ak, av)), Some((_, bv))) = (m.a.pop_front(), m.b.next()) {
            let r = (self.0)(av, bv);
            m.a.push((ak, r));
        }
    }
}

struct RightBiasedUnionOp;

impl<'a, K: Ord, V, I: MergeStateMut<(K, V), (K, V)>> MergeOperation<(K, V), (K, V), I>
    for RightBiasedUnionOp
{
    fn cmp(&self, a: &(K, V), b: &(K, V)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut I, n: usize) {
        m.move_a(n);
    }
    fn from_b(&self, m: &mut I, n: usize) {
        m.move_b(n);
    }
    fn collision(&self, m: &mut I) {
        m.move_a(1);
        m.skip_b(1);
    }
}

pub enum OuterJoinArg<A, B> {
    Left(A),
    Right(B),
    Both(A, B),
}

struct OuterJoinOp<F>(F);
struct LeftJoinOp<F>(F);
struct RightJoinOp<F>(F);
struct InnerJoinOp<F>(F);

// struct OuterJoinWithOp<F>(F);

type PairMergeState<'a, K, A, B, R> = VecMergeState<'a, (K, A), (K, B), (K, R)>;

// type InPlacePairMergeState<'a, K, A, B> = UnsafeInPlaceMergeState<(K, A), (K, B)>;

impl<K: Ord, V> FromIterator<(K, V)> for VecMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        VecMap(sort_and_dedup_by_key(
            iter.into_iter(),
            |(k, _)| k,
            Keep::Last,
        ))
    }
}

impl<K, V> From<BTreeMap<K, V>> for VecMap<K, V> {
    fn from(value: BTreeMap<K, V>) -> Self {
        let elements: Vec<(K, V)> = value.into_iter().collect();
        Self::from_sorted_vec(elements)
    }
}

impl<K: Ord, V> Extend<(K, V)> for VecMap<K, V> {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.merge_with(iter.into_iter().collect());
    }
}

impl<'a, K: Ord + Clone, A, B, R, F: Fn(OuterJoinArg<&A, &B>) -> R>
    MergeOperation<(K, A), (K, B), PairMergeState<'a, K, A, B, R>> for OuterJoinOp<F>
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

impl<'a, K: Ord + Clone, A, B, R, F: Fn(&A, Option<&B>) -> R>
    MergeOperation<(K, A), (K, B), PairMergeState<'a, K, A, B, R>> for LeftJoinOp<F>
{
    fn cmp(&self, a: &(K, A), b: &(K, B)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        for _ in 0..n {
            if let Some((k, a)) = m.a.next() {
                let res = (self.0)(a, None);
                m.r.push((k.clone(), res));
            }
        }
    }
    fn from_b(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        m.b.drop_front(n);
    }
    fn collision(&self, m: &mut PairMergeState<'a, K, A, B, R>) {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                m.r.push((k.clone(), (self.0)(a, Some(b))));
            }
        }
    }
}

impl<'a, K: Ord + Clone, A, B, R, F: Fn(Option<&A>, &B) -> R>
    MergeOperation<(K, A), (K, B), PairMergeState<'a, K, A, B, R>> for RightJoinOp<F>
{
    fn cmp(&self, a: &(K, A), b: &(K, B)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        m.a.drop_front(n);
    }
    fn from_b(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        for _ in 0..n {
            if let Some((k, b)) = m.b.next() {
                m.r.push((k.clone(), (self.0)(None, b)));
            }
        }
    }
    fn collision(&self, m: &mut PairMergeState<'a, K, A, B, R>) {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                m.r.push((k.clone(), (self.0)(Some(a), b)));
            }
        }
    }
}

impl<'a, K: Ord + Clone, A, B, R, F: Fn(&A, &B) -> R>
    MergeOperation<(K, A), (K, B), PairMergeState<'a, K, A, B, R>> for InnerJoinOp<F>
{
    fn cmp(&self, a: &(K, A), b: &(K, B)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        m.a.drop_front(n);
    }
    fn from_b(&self, m: &mut PairMergeState<'a, K, A, B, R>, n: usize) {
        m.b.drop_front(n);
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

impl<K, V> VecMap<K, V> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn as_slice(&self) -> &[(K, V)] {
        self.0.as_slice()
    }

    pub fn retain<F: FnMut((&K, &V)) -> bool>(&mut self, mut f: F) {
        self.0.retain(|entry| f((&entry.0, &entry.1)))
    }

    pub(crate) fn slice_iter(&self) -> SliceIterator<(K, V)> {
        SliceIterator(self.0.as_slice())
    }

    pub fn map_values<R, F: FnMut(V) -> R>(self, mut f: F) -> VecMap<K, R> {
        VecMap::from_sorted_vec(
            self.0
                .into_iter()
                .map(|entry| (entry.0, f(entry.1)))
                .collect(),
        )
    }

    pub(crate) fn from_sorted_vec(v: Vec<(K, V)>) -> Self {
        Self(v)
    }

    pub fn into_sorted_vec(self) -> Vec<(K, V)> {
        self.0
    }
}

impl<K: Ord, V> VecMap<K, V> {
    pub fn merge_with(&mut self, rhs: VecMap<K, V>) {
        UnsafeInPlaceMergeState::merge(&mut self.0, rhs.0, RightBiasedUnionOp)
    }

    pub fn combine_with<F: Fn(V, V) -> V>(&mut self, that: VecMap<K, V>, f: F) {
        UnsafeInPlaceMergeState::merge(&mut self.0, that.0, CombineOp(f, std::marker::PhantomData));
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let elements = self.0.as_slice();
        elements
            .binary_search_by(|p| p.0.borrow().cmp(key))
            .map(|index| &elements[index].1)
            .ok()
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
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

impl<K: Ord + Clone, V: Clone> VecMap<K, V> {
    pub fn single(k: K, v: V) -> Self {
        Self::from_sorted_vec(vec![(k, v)])
    }

    // pub fn outer_join_with<W: Clone, R, F: Fn(OuterJoinArg<&V, W>)>(
    //     &self,
    //     that: &VecMap<K, W>,
    //     f: F,
    // ) -> VecMap<K, R> {
    //     VecMap::<K, R>::from_sorted_vec(VecMergeState::merge(
    //         self.0.as_slice(),
    //         that.0.as_slice(),
    //         OuterJoinWithOp(f),
    //     ))
    // }

    pub fn outer_join<W: Clone, R, F: Fn(OuterJoinArg<&V, &W>) -> R>(
        &self,
        that: &VecMap<K, W>,
        f: F,
    ) -> VecMap<K, R> {
        VecMap::<K, R>::from_sorted_vec(VecMergeState::merge(
            self.0.as_slice(),
            that.0.as_slice(),
            OuterJoinOp(f),
        ))
    }

    pub fn left_join<W: Clone, R, F: Fn(&V, Option<&W>) -> R>(
        &self,
        that: &VecMap<K, W>,
        f: F,
    ) -> VecMap<K, R> {
        VecMap::<K, R>::from_sorted_vec(VecMergeState::merge(
            self.0.as_slice(),
            that.0.as_slice(),
            LeftJoinOp(f),
        ))
    }

    pub fn right_join<W: Clone, R, F: Fn(Option<&V>, &W) -> R>(
        &self,
        that: &VecMap<K, W>,
        f: F,
    ) -> VecMap<K, R> {
        VecMap::<K, R>::from_sorted_vec(VecMergeState::merge(
            self.0.as_slice(),
            that.0.as_slice(),
            RightJoinOp(f),
        ))
    }

    pub fn inner_join<W: Clone, R, F: Fn(&V, &W) -> R>(
        &self,
        that: &VecMap<K, W>,
        f: F,
    ) -> VecMap<K, R> {
        VecMap::<K, R>::from_sorted_vec(VecMergeState::merge(
            self.0.as_slice(),
            that.0.as_slice(),
            InnerJoinOp(f),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::btreemap;
    use quickcheck::*;
    use std::collections::BTreeMap;
    use OuterJoinArg::*;

    type Test = VecMap<i32, i32>;
    type Ref = BTreeMap<i32, i32>;

    impl<K: Arbitrary + Ord, V: Arbitrary> Arbitrary for VecMap<K, V> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let t: BTreeMap<K, V> = Arbitrary::arbitrary(g);
            t.into()
        }
    }

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
            let actual = a.inner_join(&b, |a,_| a.clone());
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
