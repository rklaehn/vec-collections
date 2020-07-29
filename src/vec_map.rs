//! A map based on a `SmallVec<(K, V)>` of key value pairs.
//!
//! An advantage of this map compared to e.g. BTreeMap is that small maps will be stored inline without allocations.
//! Larger maps will be stored using a single object on the heap.
//!
//! A disadvante is that insertion and removal of single mappings is very slow (O(N)) for large maps.
//!
use crate::binary_merge::{EarlyOut, MergeOperation};
use crate::dedup::{sort_and_dedup_by_key, Keep};
use crate::iterators::SliceIterator;
use crate::merge_state::{InPlaceMergeState, MergeStateMut, SmallVecMergeState};
use smallvec::{Array, SmallVec};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::iter::FromIterator;
use std::hash::Hash;

/// A map backed by a `SmallVec<(K, V)>`. Default inline size is 2, so maps with 0, 1 or 2 elements will not allocate.
pub struct VecMap<A: Array>(SmallVec<A>);

pub type VecMap2<K, V> = VecMap<[(K, V); 2]>;

impl<T: Debug, A: Array<Item = T>> Debug for VecMap<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.as_slice().iter()).finish()
    }   
}

impl<T: Clone, A: Array<Item = T>> Clone for VecMap<A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Hash, A: Array<Item = T>> Hash for VecMap<A> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: PartialEq, A: Array<Item = T>> PartialEq for VecMap<A> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq, A: Array<Item = T>> Eq for VecMap<A> {}

impl<T: PartialOrd, A: Array<Item = T>> PartialOrd for VecMap<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Ord, A: Array<Item = T>> Ord for VecMap<A> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<A: Array> IntoIterator for VecMap<A> {
    type Item = A::Item;
    type IntoIter = smallvec::IntoIter<A>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<A: Array> Default for VecMap<A> {
    fn default() -> Self {
        VecMap(SmallVec::default())
    }
}

struct CombineOp<F, K>(F, std::marker::PhantomData<K>);

impl<K: Ord, V, A: Array<Item = (K, V)>, F: Fn(V, V) -> V> MergeOperation<InPlaceMergeState<A, A>>
    for CombineOp<F, K>
{
    fn cmp(&self, a: &(K, V), b: &(K, V)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut InPlaceMergeState<A, A>, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut InPlaceMergeState<A, A>, n: usize) -> EarlyOut {
        m.advance_b(n, true)
    }
    fn collision(&self, m: &mut InPlaceMergeState<A, A>) -> EarlyOut {
        if let (Some((ak, av)), Some((_, bv))) = (m.a.pop_front(), m.b.next()) {
            let r = (self.0)(av, bv);
            m.a.push((ak, r));
        }
        Some(())
    }
}

struct RightBiasedUnionOp;

impl<'a, K: Ord, V, I: MergeStateMut<A = (K, V), B = (K, V)>> MergeOperation<I>
    for RightBiasedUnionOp
{
    fn cmp(&self, a: &(K, V), b: &(K, V)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, true)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.advance_a(1, false)?;
        m.advance_b(1, true)
    }
}

pub enum OuterJoinArg<K, A, B> {
    Left(K, A),
    Right(K, B),
    Both(K, A, B),
}

struct OuterJoinOp<F>(F);
struct LeftJoinOp<F>(F);
struct RightJoinOp<F>(F);
struct InnerJoinOp<F>(F);

// struct OuterJoinWithOp<F>(F);

// type InPlacePairMergeState<'a, K, A, B> = UnsafeInPlaceMergeState<(K, A), (K, B)>;

impl<K: Ord, V, A: Array<Item=(K, V)>> FromIterator<(K, V)> for VecMap<A> {
    fn from_iter<I: IntoIterator<Item = A::Item>>(iter: I) -> Self {
        VecMap(sort_and_dedup_by_key(iter.into_iter(), |(k, _)| k, Keep::Last).into())
    }
}

impl<K, V, A: Array<Item=(K, V)>> From<BTreeMap<K, V>> for VecMap<A> {
    fn from(value: BTreeMap<K, V>) -> Self {
        let elements: Vec<(K, V)> = value.into_iter().collect();
        Self::from_sorted_vec(elements)
    }
}

impl<K: Ord + 'static, V, A: Array<Item=(K, V)>> Extend<A::Item> for VecMap<A> {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.merge_with::<A>(iter.into_iter().collect());
    }
}

impl<
        'a,
        K: Ord + Clone,
        A,
        B,
        R,
        Arr: Array<Item = (K, R)>,
        F: Fn(OuterJoinArg<&K, &A, &B>) -> Option<R>,
    > MergeOperation<SmallVecMergeState<'a, (K, A), (K, B), Arr>> for OuterJoinOp<F>
{
    fn cmp(&self, a: &(K, A), b: &(K, B)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut SmallVecMergeState<'a, (K, A), (K, B), Arr>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, a)) = m.a.next() {
                let arg = OuterJoinArg::Left(k, a);
                if let Some(res) = (self.0)(arg) {
                    m.r.push((k.clone(), res));
                }
            }
        }
        Some(())
    }
    fn from_b(&self, m: &mut SmallVecMergeState<'a, (K, A), (K, B), Arr>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, b)) = m.b.next() {
                let arg = OuterJoinArg::Right(k, b);
                if let Some(res) = (self.0)(arg) {
                    m.r.push((k.clone(), res));
                }
            }
        }
        Some(())
    }
    fn collision(&self, m: &mut SmallVecMergeState<'a, (K, A), (K, B), Arr>) -> EarlyOut {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                let arg = OuterJoinArg::Both(k, a, b);
                if let Some(res) = (self.0)(arg) {
                    m.r.push((k.clone(), res));
                }
            }
        }
        Some(())
    }
}

impl<
        'a,
        K: Ord + Clone,
        A,
        B,
        R,
        Arr: Array<Item = (K, R)>,
        F: Fn(&K, &A, Option<&B>) -> Option<R>,
    > MergeOperation<SmallVecMergeState<'a, (K, A), (K, B), Arr>> for LeftJoinOp<F>
{
    fn cmp(&self, a: &(K, A), b: &(K, B)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut SmallVecMergeState<'a, (K, A), (K, B), Arr>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, a)) = m.a.next() {
                if let Some(res) = (self.0)(k, a, None) {
                    m.r.push((k.clone(), res));
                }
            }
        }
        Some(())
    }
    fn from_b(&self, m: &mut SmallVecMergeState<'a, (K, A), (K, B), Arr>, n: usize) -> EarlyOut {
        m.b.drop_front(n);
        Some(())
    }
    fn collision(&self, m: &mut SmallVecMergeState<'a, (K, A), (K, B), Arr>) -> EarlyOut {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                if let Some(res) = (self.0)(k, a, Some(b)) {
                    m.r.push((k.clone(), res));
                }
            }
        }
        Some(())
    }
}

impl<
        'a,
        K: Ord + Clone,
        A,
        B,
        R,
        Arr: Array<Item = (K, R)>,
        F: Fn(&K, Option<&A>, &B) -> Option<R>,
    > MergeOperation<SmallVecMergeState<'a, (K, A), (K, B), Arr>> for RightJoinOp<F>
{
    fn cmp(&self, a: &(K, A), b: &(K, B)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut SmallVecMergeState<'a, (K, A), (K, B), Arr>, n: usize) -> EarlyOut {
        m.a.drop_front(n);
        Some(())
    }
    fn from_b(&self, m: &mut SmallVecMergeState<'a, (K, A), (K, B), Arr>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, b)) = m.b.next() {
                if let Some(res) = (self.0)(k, None, b) {
                    m.r.push((k.clone(), res));
                }
            }
        }
        Some(())
    }
    fn collision(&self, m: &mut SmallVecMergeState<'a, (K, A), (K, B), Arr>) -> EarlyOut {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                if let Some(res) = (self.0)(k, Some(a), b) {
                    m.r.push((k.clone(), res));
                }
            }
        }
        Some(())
    }
}

impl<'a, K: Ord + Clone, A, B, R, Arr: Array<Item = (K, R)>, F: Fn(&K, &A, &B) -> Option<R>>
    MergeOperation<SmallVecMergeState<'a, (K, A), (K, B), Arr>> for InnerJoinOp<F>
{
    fn cmp(&self, a: &(K, A), b: &(K, B)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut SmallVecMergeState<'a, (K, A), (K, B), Arr>, n: usize) -> EarlyOut {
        m.a.drop_front(n);
        Some(())
    }
    fn from_b(&self, m: &mut SmallVecMergeState<'a, (K, A), (K, B), Arr>, n: usize) -> EarlyOut {
        m.b.drop_front(n);
        Some(())
    }
    fn collision(&self, m: &mut SmallVecMergeState<'a, (K, A), (K, B), Arr>) -> EarlyOut {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                if let Some(res) = (self.0)(k, a, b) {
                    m.r.push((k.clone(), res));
                }
            }
        }
        Some(())
    }
}

impl<A: Array> VecMap<A> {
    /// private because it does not check invariants
    pub(crate) fn new(value: SmallVec<A>) -> Self {
        Self(value)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn empty() -> Self {
        Self(SmallVec::new())
    }

    /// number of mappings
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// the underlying memory as a slice of key value pairs
    pub fn as_slice(&self) -> &[A::Item] {
        self.0.as_slice()
    }

    /// retain all pairs matching a predicate
    pub fn retain<F: FnMut(&A::Item) -> bool>(&mut self, mut f: F) {
        self.0.retain(|entry| f(entry))
    }

    pub(crate) fn slice_iter(&self) -> SliceIterator<A::Item> {
        SliceIterator(self.0.as_slice())
    }

    // /// map values while keeping keys
    // pub fn map_values<R, F: FnMut(V) -> R>(self, mut f: F) -> VecMap<K, R> {
    //     VecMap::from_sorted_vec(
    //         self.0
    //             .into_iter()
    //             .map(|entry| (entry.0, f(entry.1)))
    //             .collect(),
    //     )
    // }

    pub(crate) fn from_sorted_vec(v: Vec<A::Item>) -> Self {
        Self(v.into())
    }

    pub fn into_sorted_vec(self) -> SmallVec<A> {
        self.0
    }
}

impl<A: Array> AsRef<[A::Item]> for VecMap<A> {
    fn as_ref(&self) -> &[A::Item] {
        self.as_slice()
    }

}

impl<K: Ord + 'static, V, A: Array<Item = (K, V)>> VecMap<A> {
    /// in-place merge with another map of the same type. The merge is right-biased, so on collisions the values
    /// from the rhs will win.
    pub fn merge_with<B: Array<Item = (K, V)>>(&mut self, rhs: VecMap<B>) {
        InPlaceMergeState::merge(&mut self.0, rhs.0, RightBiasedUnionOp)
    }

    /// in-place combine with another map of the same type. The given function allows to select the value in case
    /// of collisions.
    pub fn combine_with<F: Fn(V, V) -> V>(&mut self, that: VecMap<A>, f: F) {
        InPlaceMergeState::merge(&mut self.0, that.0, CombineOp(f, std::marker::PhantomData));
    }

    /// lookup of a mapping. Time complexity is O(log N). Binary search.
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

impl<K: Ord + Clone, V: Clone, A: Array<Item = (K, V)>> VecMap<A> {
    // pub fn outer_join_with<W: Clone, R, F: Fn(OuterJoinArg<&K, &V, &W>)>(
    //     &self,
    //     that: &VecMap2<K, W>,
    //     f: F,
    // ) -> VecMap2<K, R> {
    //     VecMap2::<K, R>::from_sorted_vec(VecMergeState::merge(
    //         self.0.as_slice(),
    //         that.0.as_slice(),
    //         OuterJoinWithOp(f),
    //     ))
    // }

    pub fn outer_join<W: Clone, R, F: Fn(OuterJoinArg<&K, &V, &W>) -> Option<R>, B: Array<Item = (K, W)>>(
        &self,
        that: &VecMap<B>,
        f: F,
    ) -> VecMap2<K, R> {
        VecMap2::<K, R>::new(SmallVecMergeState::merge(
            self.0.as_slice(),
            that.0.as_slice(),
            OuterJoinOp(f),
        ))
    }

    pub fn left_join<W: Clone, R, F: Fn(&K, &V, Option<&W>) -> Option<R>, B: Array<Item = (K, W)>>(
        &self,
        that: &VecMap2<K, W>,
        f: F,
    ) -> VecMap2<K, R> {
        VecMap2::<K, R>::new(SmallVecMergeState::merge(
            self.0.as_slice(),
            that.0.as_slice(),
            LeftJoinOp(f),
        ))
    }

    pub fn right_join<W: Clone, R, F: Fn(&K, Option<&V>, &W) -> Option<R>, B: Array<Item = (K, W)>>(
        &self,
        that: &VecMap<B>,
        f: F,
    ) -> VecMap2<K, R> {
        VecMap2::<K, R>::new(SmallVecMergeState::merge(
            self.0.as_slice(),
            that.0.as_slice(),
            RightJoinOp(f),
        ))
    }

    pub fn inner_join<W: Clone, R, F: Fn(&K, &V, &W) -> Option<R>, B: Array<Item = (K, W)>>(
        &self,
        that: &VecMap<B>,
        f: F,
    ) -> VecMap2<K, R> {
        VecMap2::<K, R>::new(SmallVecMergeState::merge(
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

    type Test = VecMap2<i32, i32>;
    type Ref = BTreeMap<i32, i32>;

    impl<K: Arbitrary + Ord, V: Arbitrary> Arbitrary for VecMap2<K, V> {
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
            let actual = a.outer_join(&b, |arg| Some(match arg {
                Left(_, a) => a.clone(),
                Right(_, b) => b.clone(),
                Both(_, _, b) => b.clone(),
            }));
            expected == actual
        }

        fn inner_join(a: Ref, b: Ref) -> bool {
            let expected: Test = inner_join_reference(&a, &b).into();
            let a: Test = a.into();
            let b: Test = b.into();
            let actual = a.inner_join(&b, |_, a,_| Some(a.clone()));
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
        let actual = a.outer_join(&b, |arg| {
            Some(match arg {
                Left(_, a) => a.clone(),
                Right(_, b) => b.clone(),
                Both(_, _, b) => b.clone(),
            })
        });
        assert_eq!(actual, expected);
        println!("{:?}", actual);
    }
}
