#[cfg(feature = "total")]
use crate::iterators::SliceIterator;
use crate::{
    binary_merge::{EarlyOut, MergeOperation},
    dedup::{sort_and_dedup_by_key, Keep},
    merge_state::{InPlaceMergeStateRef, MergeStateMut, SmallVecMergeState},
    VecSet,
};
use crate::{iterators::VecMapIter, merge_state::InPlaceMergeState};
use core::{borrow::Borrow, cmp::Ordering, fmt, fmt::Debug, hash, hash::Hash, iter::FromIterator};
use smallvec::{Array, SmallVec};
use std::collections::BTreeMap;
#[cfg(feature = "serde")]
use {
    core::marker::PhantomData,
    serde::{
        de::{Deserialize, Deserializer, MapAccess, Visitor},
        ser::{Serialize, SerializeMap, Serializer},
    },
};

/// An abstract vec map
pub trait AbstractVecMap<K, V> {
    fn as_slice(&self) -> &[(K, V)];

    fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    fn iter(&self) -> VecMapIter<core::slice::Iter<(K, V)>> {
        VecMapIter::new(self.as_slice().iter())
    }

    /// lookup of a mapping. Time complexity is O(log N). Binary search.
    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + 'static,
        Q: Ord + ?Sized,
    {
        let elements = self.as_slice();
        elements
            .binary_search_by(|p| p.0.borrow().cmp(key))
            .map(|index| &elements[index].1)
            .ok()
    }

    /// Perform an outer join with another VecMap, producing a new result
    ///
    ///
    fn outer_join<W, R, F, A>(&self, that: &impl AbstractVecMap<K, W>, f: F) -> VecMap<A>
    where
        K: Ord + Clone,
        A: Array<Item = (K, R)>,
        F: Fn(OuterJoinArg<&K, &V, &W>) -> Option<R>,
    {
        VecMap::<A>::new(SmallVecMergeState::merge(
            self.as_slice(),
            that.as_slice(),
            OuterJoinOp(f),
        ))
    }

    fn left_join<W, R, F, A>(&self, that: &impl AbstractVecMap<K, W>, f: F) -> VecMap<A>
    where
        K: Ord + Clone,
        F: Fn(&K, &V, Option<&W>) -> Option<R>,
        A: Array<Item = (K, R)>,
    {
        VecMap::new(SmallVecMergeState::merge(
            self.as_slice(),
            that.as_slice(),
            LeftJoinOp(f),
        ))
    }

    fn right_join<W, R, F, A>(&self, that: &impl AbstractVecMap<K, W>, f: F) -> VecMap<A>
    where
        K: Ord + Clone,
        F: Fn(&K, Option<&V>, &W) -> Option<R>,
        A: Array<Item = (K, R)>,
    {
        VecMap::new(SmallVecMergeState::merge(
            self.as_slice(),
            that.as_slice(),
            RightJoinOp(f),
        ))
    }

    fn inner_join<W, R, F, A>(&self, that: &impl AbstractVecMap<K, W>, f: F) -> VecMap<A>
    where
        K: Ord + Clone,
        F: Fn(&K, &V, &W) -> Option<R>,
        A: Array<Item = (K, R)>,
    {
        VecMap::new(SmallVecMergeState::merge(
            self.as_slice(),
            that.as_slice(),
            InnerJoinOp(f),
        ))
    }
}

impl<K, V, A: Array<Item = (K, V)>> AbstractVecMap<K, V> for VecMap<A> {
    fn as_slice(&self) -> &[A::Item] {
        self.0.as_slice()
    }
}

#[cfg(feature = "rkyv")]
impl<K, V> AbstractVecMap<K, V> for ArchivedVecMap<K, V> {
    fn as_slice(&self) -> &[(K, V)] {
        self.0.as_slice()
    }
}

/// A map backed by a [SmallVec] of key value pairs.
///
/// [SmallVec]: https://docs.rs/smallvec/1.4.1/smallvec/struct.SmallVec.html
pub struct VecMap<A: Array>(SmallVec<A>);

/// Type alias for a [VecMap](struct.VecMap) with up to 1 mapping with inline storage.
///
/// This is a good default, since for usize sized keys and values, 1 mapping is the max you can fit in without making the struct larger.
pub type VecMap1<K, V> = VecMap<[(K, V); 1]>;

impl<T: Debug, A: Array<Item = T>> Debug for VecMap<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.as_slice().iter()).finish()
    }
}

impl<T: Clone, A: Array<Item = T>> Clone for VecMap<A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Hash, A: Array<Item = T>> Hash for VecMap<A> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
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

impl<'a, K: 'a, V: 'a, A: Array<Item = (K, V)>> IntoIterator for &'a VecMap<A> {
    type Item = &'a A::Item;
    type IntoIter = VecMapIter<core::slice::Iter<'a, A::Item>>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<A: Array> IntoIterator for VecMap<A> {
    type Item = A::Item;
    type IntoIter = VecMapIter<smallvec::IntoIter<A>>;
    fn into_iter(self) -> Self::IntoIter {
        VecMapIter::new(self.0.into_iter())
    }
}

impl<A: Array> Default for VecMap<A> {
    fn default() -> Self {
        VecMap(SmallVec::default())
    }
}

impl<A: Array> From<VecMap<A>> for VecSet<A> {
    fn from(value: VecMap<A>) -> Self {
        // entries are sorted by unique first elemnt, so they are also a valid set
        VecSet::new_unsafe(value.0)
    }
}

struct CombineOp<F, K>(F, std::marker::PhantomData<K>);

impl<'a, K: Ord, V, A: Array<Item = (K, V)>, B: Array<Item = (K, V)>, F: Fn(V, V) -> V>
    MergeOperation<InPlaceMergeState<'a, A, B>> for CombineOp<F, K>
{
    fn cmp(&self, a: &(K, V), b: &(K, V)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut InPlaceMergeState<A, B>, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut InPlaceMergeState<A, B>, n: usize) -> EarlyOut {
        m.advance_b(n, true)
    }
    fn collision(&self, m: &mut InPlaceMergeState<A, B>) -> EarlyOut {
        if let (Some((ak, av)), Some((_, bv))) = (m.a.pop_front(), m.b.next()) {
            let r = (self.0)(av, bv);
            m.a.push((ak, r));
        }
        Some(())
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

impl<K: Ord, V, A: Array<Item = (K, V)>> FromIterator<(K, V)> for VecMap<A> {
    fn from_iter<I: IntoIterator<Item = A::Item>>(iter: I) -> Self {
        VecMap(sort_and_dedup_by_key(iter.into_iter(), |(k, _)| k, Keep::Last).into())
    }
}

impl<K, V, A: Array<Item = (K, V)>> From<BTreeMap<K, V>> for VecMap<A> {
    fn from(value: BTreeMap<K, V>) -> Self {
        Self::new(value.into_iter().collect())
    }
}

impl<K: Ord + 'static, V, A: Array<Item = (K, V)>> Extend<A::Item> for VecMap<A> {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.merge_with::<A>(iter.into_iter().collect());
    }
}

impl<A: Array> AsRef<[A::Item]> for VecMap<A> {
    fn as_ref(&self) -> &[A::Item] {
        self.as_slice()
    }
}

impl<A: Array> From<VecMap<A>> for SmallVec<A> {
    fn from(value: VecMap<A>) -> Self {
        value.0
    }
}

impl<'a, K, V, W, R, A, F> MergeOperation<SmallVecMergeState<'a, (K, V), (K, W), A>>
    for OuterJoinOp<F>
where
    K: Ord + Clone,
    A: Array<Item = (K, R)>,
    F: Fn(OuterJoinArg<&K, &V, &W>) -> Option<R>,
{
    fn cmp(&self, a: &(K, V), b: &(K, W)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut SmallVecMergeState<'a, (K, V), (K, W), A>, n: usize) -> EarlyOut {
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
    fn from_b(&self, m: &mut SmallVecMergeState<'a, (K, V), (K, W), A>, n: usize) -> EarlyOut {
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
    fn collision(&self, m: &mut SmallVecMergeState<'a, (K, V), (K, W), A>) -> EarlyOut {
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

impl<'a, K, V, W, F, A> MergeOperation<InPlaceMergeStateRef<'a, A, (K, W)>> for OuterJoinOp<F>
where
    A: Array<Item = (K, V)>,
    K: Ord + Clone,
    F: Fn(OuterJoinArg<&K, V, &W>) -> Option<V>,
{
    fn cmp(&self, a: &(K, V), b: &(K, W)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut InPlaceMergeStateRef<'a, A, (K, W)>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, v)) = m.a.pop_front() {
                if let Some(v) = (self.0)(OuterJoinArg::Left(&k, v)) {
                    m.a.push((k, v));
                }
            }
        }
        Some(())
    }
    fn from_b(&self, m: &mut InPlaceMergeStateRef<'a, A, (K, W)>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, b)) = m.b.next() {
                if let Some(v) = (self.0)(OuterJoinArg::Right(k, b)) {
                    m.a.push((k.clone(), v));
                }
            }
        }
        Some(())
    }
    fn collision(&self, m: &mut InPlaceMergeStateRef<'a, A, (K, W)>) -> EarlyOut {
        if let Some((k, v)) = m.a.pop_front() {
            if let Some((_, w)) = m.b.next() {
                if let Some(v) = (self.0)(OuterJoinArg::Both(&k, v, w)) {
                    m.a.push((k, v));
                }
            }
        }
        Some(())
    }
}

impl<'a, K, V, W, F, A, B> MergeOperation<InPlaceMergeState<'a, A, B>> for OuterJoinOp<F>
where
    A: Array<Item = (K, V)>,
    B: Array<Item = (K, W)>,
    K: Ord,
    F: Fn(OuterJoinArg<&K, V, W>) -> Option<V>,
{
    fn cmp(&self, a: &(K, V), b: &(K, W)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut InPlaceMergeState<'a, A, B>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, v)) = m.a.pop_front() {
                if let Some(v) = (self.0)(OuterJoinArg::Left(&k, v)) {
                    m.a.push((k, v));
                }
            }
        }
        Some(())
    }
    fn from_b(&self, m: &mut InPlaceMergeState<'a, A, B>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, b)) = m.b.next() {
                if let Some(v) = (self.0)(OuterJoinArg::Right(&k, b)) {
                    m.a.push((k, v));
                }
            }
        }
        Some(())
    }
    fn collision(&self, m: &mut InPlaceMergeState<'a, A, B>) -> EarlyOut {
        if let Some((k, v)) = m.a.pop_front() {
            if let Some((_, w)) = m.b.next() {
                if let Some(v) = (self.0)(OuterJoinArg::Both(&k, v, w)) {
                    m.a.push((k, v));
                }
            }
        }
        Some(())
    }
}

impl<'a, K, V, W, R, F, A> MergeOperation<SmallVecMergeState<'a, (K, V), (K, W), A>>
    for LeftJoinOp<F>
where
    K: Ord + Clone,
    A: Array<Item = (K, R)>,
    F: Fn(&K, &V, Option<&W>) -> Option<R>,
{
    fn cmp(&self, a: &(K, V), b: &(K, W)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut SmallVecMergeState<'a, (K, V), (K, W), A>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, a)) = m.a.next() {
                if let Some(res) = (self.0)(k, a, None) {
                    m.r.push((k.clone(), res));
                }
            }
        }
        Some(())
    }
    fn from_b(&self, m: &mut SmallVecMergeState<'a, (K, V), (K, W), A>, n: usize) -> EarlyOut {
        m.b.drop_front(n);
        Some(())
    }
    fn collision(&self, m: &mut SmallVecMergeState<'a, (K, V), (K, W), A>) -> EarlyOut {
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

impl<'a, K, V, W, F, A> MergeOperation<InPlaceMergeStateRef<'a, A, (K, W)>> for LeftJoinOp<F>
where
    A: Array<Item = (K, V)>,
    K: Ord + Clone,
    F: Fn(&K, V, Option<&W>) -> Option<V>,
{
    fn cmp(&self, a: &(K, V), b: &(K, W)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut InPlaceMergeStateRef<'a, A, (K, W)>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, v)) = m.a.pop_front() {
                if let Some(v) = (self.0)(&k, v, None) {
                    m.a.push((k, v))
                }
            }
        }
        Some(())
    }
    fn from_b(&self, m: &mut InPlaceMergeStateRef<'a, A, (K, W)>, n: usize) -> EarlyOut {
        m.b.drop_front(n);
        Some(())
    }
    fn collision(&self, m: &mut InPlaceMergeStateRef<'a, A, (K, W)>) -> EarlyOut {
        if let Some((k, v)) = m.a.pop_front() {
            if let Some((_, w)) = m.b.next() {
                if let Some(v) = (self.0)(&k, v, Some(w)) {
                    m.a.push((k, v))
                }
            }
        }
        Some(())
    }
}

impl<'a, K, V, W, R, F, A> MergeOperation<SmallVecMergeState<'a, (K, V), (K, W), A>>
    for RightJoinOp<F>
where
    K: Ord + Clone,
    A: Array<Item = (K, R)>,
    F: Fn(&K, Option<&V>, &W) -> Option<R>,
{
    fn cmp(&self, a: &(K, V), b: &(K, W)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut SmallVecMergeState<'a, (K, V), (K, W), A>, n: usize) -> EarlyOut {
        m.a.drop_front(n);
        Some(())
    }
    fn from_b(&self, m: &mut SmallVecMergeState<'a, (K, V), (K, W), A>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, b)) = m.b.next() {
                if let Some(res) = (self.0)(k, None, b) {
                    m.r.push((k.clone(), res));
                }
            }
        }
        Some(())
    }
    fn collision(&self, m: &mut SmallVecMergeState<'a, (K, V), (K, W), A>) -> EarlyOut {
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

impl<'a, K, V, W, F, A> MergeOperation<InPlaceMergeStateRef<'a, A, (K, W)>> for RightJoinOp<F>
where
    A: Array<Item = (K, V)>,
    K: Ord + Clone,
    F: Fn(&K, Option<V>, &W) -> Option<V>,
{
    fn cmp(&self, a: &(K, V), b: &(K, W)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut InPlaceMergeStateRef<'a, A, (K, W)>, n: usize) -> EarlyOut {
        m.a.consume(n, false);
        Some(())
    }
    fn from_b(&self, m: &mut InPlaceMergeStateRef<'a, A, (K, W)>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, w)) = m.b.next() {
                if let Some(v) = (self.0)(k, None, w) {
                    m.a.push((k.clone(), v))
                }
            }
        }
        Some(())
    }
    fn collision(&self, m: &mut InPlaceMergeStateRef<'a, A, (K, W)>) -> EarlyOut {
        if let Some((k, v)) = m.a.pop_front() {
            if let Some((_, w)) = m.b.next() {
                if let Some(res) = (self.0)(&k, Some(v), w) {
                    m.a.push((k, res));
                }
            }
        }
        Some(())
    }
}

impl<'a, K, V, W, R, F, A> MergeOperation<SmallVecMergeState<'a, (K, V), (K, W), A>>
    for InnerJoinOp<F>
where
    K: Ord + Clone,
    A: Array<Item = (K, R)>,
    F: Fn(&K, &V, &W) -> Option<R>,
{
    fn cmp(&self, a: &(K, V), b: &(K, W)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut SmallVecMergeState<'a, (K, V), (K, W), A>, n: usize) -> EarlyOut {
        m.a.drop_front(n);
        Some(())
    }
    fn from_b(&self, m: &mut SmallVecMergeState<'a, (K, V), (K, W), A>, n: usize) -> EarlyOut {
        m.b.drop_front(n);
        Some(())
    }
    fn collision(&self, m: &mut SmallVecMergeState<'a, (K, V), (K, W), A>) -> EarlyOut {
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

impl<'a, K, V, W, F, A> MergeOperation<InPlaceMergeStateRef<'a, A, (K, W)>> for InnerJoinOp<F>
where
    A: Array<Item = (K, V)>,
    K: Ord + Clone,
    F: Fn(&K, V, &W) -> Option<V>,
{
    fn cmp(&self, a: &(K, V), b: &(K, W)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut InPlaceMergeStateRef<'a, A, (K, W)>, n: usize) -> EarlyOut {
        m.a.consume(n, false);
        Some(())
    }
    fn from_b(&self, m: &mut InPlaceMergeStateRef<'a, A, (K, W)>, n: usize) -> EarlyOut {
        m.b.drop_front(n);
        Some(())
    }
    fn collision(&self, m: &mut InPlaceMergeStateRef<'a, A, (K, W)>) -> EarlyOut {
        if let Some((k, v)) = m.a.pop_front() {
            if let Some((_, w)) = m.b.next() {
                if let Some(v) = (self.0)(&k, v, w) {
                    m.a.push((k, v))
                }
            }
        }
        Some(())
    }
}

impl<K, V, A: Array<Item = (K, V)>> VecMap<A> {
    /// map values while keeping keys
    pub fn map_values<R, B: Array<Item = (K, R)>, F: FnMut(V) -> R>(self, mut f: F) -> VecMap<B> {
        VecMap::new(
            self.0
                .into_iter()
                .map(|entry| (entry.0, f(entry.1)))
                .collect(),
        )
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
    fn as_slice(&self) -> &[A::Item] {
        self.0.as_ref()
    }

    /// retain all pairs matching a predicate
    pub fn retain<F: FnMut(&A::Item) -> bool>(&mut self, mut f: F) {
        self.0.retain(|entry| f(entry))
    }

    #[cfg(feature = "total")]
    pub(crate) fn slice_iter(&self) -> SliceIterator<A::Item> {
        SliceIterator(self.0.as_slice())
    }

    pub fn into_inner(self) -> SmallVec<A> {
        self.0
    }

    /// Creates a vecmap with a single item
    pub fn single(item: A::Item) -> Self {
        Self(smallvec::smallvec![item])
    }
}

impl<K: Ord + 'static, V, A: Array<Item = (K, V)>> VecMap<A> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.0.binary_search_by(|(k, _)| k.cmp(&key)) {
            Ok(index) => {
                let mut elem = (key, value);
                std::mem::swap(&mut elem, &mut self.0[index]);
                Some(elem.1)
            }
            Err(ip) => {
                self.0.insert(ip, (key, value));
                None
            }
        }
    }

    fn inner_join_with<W, F>(&mut self, that: &impl AbstractVecMap<K, W>, f: F)
    where
        K: Ord + Clone,
        F: Fn(&K, V, &W) -> Option<V>,
    {
        InPlaceMergeStateRef::merge(&mut self.0, &that.as_slice(), InnerJoinOp(f))
    }

    fn left_join_with<W, F>(&mut self, that: &impl AbstractVecMap<K, W>, f: F)
    where
        K: Ord + Clone,
        F: Fn(&K, V, Option<&W>) -> Option<V>,
    {
        InPlaceMergeStateRef::merge(&mut self.0, &that.as_slice(), LeftJoinOp(f))
    }

    fn right_join_with<W, F>(&mut self, that: &impl AbstractVecMap<K, W>, f: F)
    where
        K: Ord + Clone,
        F: Fn(&K, Option<V>, &W) -> Option<V>,
    {
        InPlaceMergeStateRef::merge(&mut self.0, &that.as_slice(), RightJoinOp(f))
    }

    fn outer_join_with<W, F>(&mut self, that: &impl AbstractVecMap<K, W>, f: F)
    where
        K: Ord + Clone,
        F: Fn(OuterJoinArg<&K, V, &W>) -> Option<V>,
    {
        InPlaceMergeStateRef::merge(&mut self.0, &that.as_slice(), OuterJoinOp(f))
    }

    /// in-place merge with another map of the same type. The merge is right-biased, so on collisions the values
    /// from the rhs will win.
    pub fn merge_with<B: Array<Item = (K, V)>>(&mut self, that: VecMap<B>) {
        self.combine_with(that, |_, r| r)
    }

    /// in-place combine with another map of the same type. The given function allows to select the value in case
    /// of collisions.
    pub fn combine_with<B: Array<Item = A::Item>, F: Fn(V, V) -> V>(
        &mut self,
        that: VecMap<B>,
        f: F,
    ) {
        InPlaceMergeState::merge(&mut self.0, that.0, OuterJoinOp(move |arg: OuterJoinArg<&K, V, V>| {
            Some(match arg {
                OuterJoinArg::Left(_, v) => v,
                OuterJoinArg::Right(_, v) => v,
                OuterJoinArg::Both(_, v, w) => f(v, w),
            })
        }));
    }
}

impl<K: Ord + 'static, V, A: Array<Item = (K, V)>> VecMap<A> {
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

#[cfg(feature = "serde")]
impl<K, V, A: Array<Item = (K, V)>> Serialize for VecMap<A>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self.0.iter() {
            state.serialize_entry(&k, &v)?;
        }
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V, A: Array<Item = (K, V)>> Deserialize<'de> for VecMap<A>
where
    K: Deserialize<'de> + Ord + PartialEq + Clone,
    V: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(VecMapVisitor {
            phantom: PhantomData,
        })
    }
}

#[cfg(feature = "serde")]
struct VecMapVisitor<K, V, A> {
    phantom: PhantomData<(K, V, A)>,
}

#[cfg(feature = "serde")]
impl<'de, K, V, A> Visitor<'de> for VecMapVisitor<K, V, A>
where
    A: Array<Item = (K, V)>,
    K: Deserialize<'de> + Ord + PartialEq + Clone,
    V: Deserialize<'de>,
{
    type Value = VecMap<A>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let len = map.size_hint().unwrap_or(0);
        let mut values: SmallVec<A> = SmallVec::with_capacity(len);

        while let Some(value) = map.next_entry::<K, V>()? {
            values.push(value);
        }
        values.sort_by_key(|x: &(K, V)| x.0.clone());
        values.dedup_by_key(|x: &mut (K, V)| x.0.clone());
        Ok(VecMap(values))
    }
}

#[cfg(feature = "rkyv")]
#[repr(transparent)]
pub struct ArchivedVecMap<K, V>(rkyv::vec::ArchivedVec<(K, V)>);

#[cfg(feature = "rkyv")]
impl<K, V, A> rkyv::Archive for VecMap<A>
where
    A: Array<Item = (K, V)>,
    K: rkyv::Archive,
    V: rkyv::Archive,
{
    type Archived = ArchivedVecMap<K::Archived, V::Archived>;

    type Resolver = rkyv::vec::VecResolver;

    unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        rkyv::vec::ArchivedVec::resolve_from_slice(self.0.as_slice(), pos, resolver, &mut (*out).0);
    }
}

#[cfg(feature = "rkyv")]
impl<S, K, V, A> rkyv::Serialize<S> for VecMap<A>
where
    A: Array<Item = (K, V)>,
    K: rkyv::Archive + rkyv::Serialize<S>,
    V: rkyv::Archive + rkyv::Serialize<S>,
    S: rkyv::ser::ScratchSpace + rkyv::ser::Serializer,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        rkyv::vec::ArchivedVec::serialize_from_slice(self.0.as_ref(), serializer)
    }
}

#[cfg(feature = "rkyv")]
impl<D, K, V, A> rkyv::Deserialize<VecMap<A>, D> for ArchivedVecMap<K::Archived, V::Archived>
where
    A: Array<Item = (K, V)>,
    K: rkyv::Archive,
    V: rkyv::Archive,
    D: rkyv::Fallible + ?Sized,
    [<<A as Array>::Item as rkyv::Archive>::Archived]:
        rkyv::DeserializeUnsized<[<A as Array>::Item], D>,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<VecMap<A>, D::Error> {
        // todo: replace this with SmallVec once smallvec support for rkyv lands on crates.io
        let items: Vec<(K, V)> = self.0.deserialize(deserializer)?;
        Ok(VecMap(items.into()))
    }
}

/// Validation error for a range set
#[cfg(feature = "rkyv_validated")]
#[derive(Debug)]
pub enum ArchivedVecMapError {
    /// error with the individual elements of the VecSet
    ValueCheckError,
    /// elements were not properly ordered
    OrderCheckError,
}

#[cfg(feature = "rkyv_validated")]
impl std::error::Error for ArchivedVecMapError {}

#[cfg(feature = "rkyv_validated")]
impl std::fmt::Display for ArchivedVecMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "rkyv_validated")]
impl<C: ?Sized, K, V> bytecheck::CheckBytes<C> for ArchivedVecMap<K, V>
where
    K: Ord,
    bool: bytecheck::CheckBytes<C>,
    rkyv::vec::ArchivedVec<(K, V)>: bytecheck::CheckBytes<C>,
{
    type Error = ArchivedVecMapError;
    unsafe fn check_bytes<'a>(
        value: *const Self,
        context: &mut C,
    ) -> Result<&'a Self, Self::Error> {
        let values = &(*value).0;
        rkyv::vec::ArchivedVec::<(K, V)>::check_bytes(values, context)
            .map_err(|_| ArchivedVecMapError::ValueCheckError)?;
        if !values
            .iter()
            .zip(values.iter().skip(1))
            .all(|((ak, _), (bk, _))| ak < bk)
        {
            return Err(ArchivedVecMapError::OrderCheckError);
        };
        Ok(&*value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::btreemap;
    use quickcheck::*;
    use std::collections::BTreeMap;
    use OuterJoinArg::*;

    type Test = VecMap1<i32, i32>;
    type Ref = BTreeMap<i32, i32>;

    impl<K: Arbitrary + Ord, V: Arbitrary> Arbitrary for VecMap1<K, V> {
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

        #[cfg(feature = "serde")]
        fn serde_roundtrip(reference: Test) -> bool {
            let bytes = serde_json::to_vec(&reference).unwrap();
            let deser = serde_json::from_slice(&bytes).unwrap();
            reference == deser
        }

        #[cfg(feature = "rkyv")]
        fn rkyv_roundtrip_unvalidated(a: Test) -> bool {
            use rkyv::*;
            use ser::Serializer;
            let mut serializer = ser::serializers::AllocSerializer::<256>::default();
            serializer.serialize_value(&a).unwrap();
            let bytes = serializer.into_serializer().into_inner();
            let archived = unsafe { rkyv::archived_root::<Test>(&bytes) };
            let deserialized: Test = archived.deserialize(&mut Infallible).unwrap();
            a == deserialized
        }

        #[cfg(feature = "rkyv_validated")]
        #[quickcheck]
        fn rkyv_roundtrip_validated(a: Test) -> bool {
            use rkyv::*;
            use ser::Serializer;
            let mut serializer = ser::serializers::AllocSerializer::<256>::default();
            serializer.serialize_value(&a).unwrap();
            let bytes = serializer.into_serializer().into_inner();
            let archived = rkyv::check_archived_root::<Test>(&bytes).unwrap();
            let deserialized: Test = archived.deserialize(&mut Infallible).unwrap();
            a == deserialized
        }

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
