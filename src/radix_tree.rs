use std::{cmp::Ordering, fmt::Debug};

use smallvec::SmallVec;

use crate::{
    binary_merge::{EarlyOut, MergeOperation},
    merge_state::{
        BoolOpMergeState, InPlaceMergeStateRef, MergeStateMut, MutateInput, SmallVecMergeState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Fragment<T>(SmallVec<[T; 16]>);

impl<T> AsRef<[T]> for Fragment<T> {
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T> std::ops::Deref for Fragment<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_ref()
    }
}

impl<'a, T: Clone> From<&'a [T]> for Fragment<T> {
    fn from(value: &'a [T]) -> Self {
        Self(value.into())
    }
}
impl<T> From<SmallVec<[T; 16]>> for Fragment<T> {
    fn from(value: SmallVec<[T; 16]>) -> Self {
        Self(value)
    }
}

impl<T: Ord> Ord for Fragment<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0[0].cmp(&other.0[0])
    }
}

impl<T: Ord> PartialOrd for Fragment<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Default for Fragment<T> {
    fn default() -> Self {
        Self(SmallVec::new())
    }
}

fn common_prefix<'a, T: Eq>(a: &'a [T], b: &'a [T]) -> usize {
    let max = a.len().min(b.len());
    for i in 0..max {
        if a[i] != b[i] {
            return i;
        }
    }
    max
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadixTree<K, V> {
    prefix: Fragment<K>,
    value: Option<V>,
    children: Vec<Self>,
}

impl<K: Clone, V> Default for RadixTree<K, V> {
    fn default() -> Self {
        Self {
            prefix: Fragment::default(),
            value: None,
            children: Vec::new(),
        }
    }
}

impl<K: Ord + Copy + Debug, V: Debug> RadixTree<K, V> {
    pub fn prefix(&self) -> &[K] {
        &self.prefix
    }

    pub fn value(&self) -> &Option<V> {
        &self.value
    }

    pub fn children(&self) -> &[Self] {
        &self.children
    }

    pub fn contains_key(&self, key: &[K]) -> bool {
        self.intersects(&RadixTree::single(key, ()))
    }

    pub fn is_subset<W: Debug>(&self, that: &RadixTree<K, W>) -> bool {
        Self::is_subset0(self, self.prefix(), that, that.prefix())
    }

    fn is_subset0<W: Debug>(l: &Self, l_prefix: &[K], r: &RadixTree<K, W>, r_prefix: &[K]) -> bool {
        let n = common_prefix(&l_prefix, &r_prefix);
        if n == l_prefix.len() && n == r_prefix.len() {
            // prefixes are identical
            (!l.value().is_some() || r.value().is_some())
                && !BoolOpMergeState::merge(l.children(), r.children(), NonSubsetOp)
        } else if n == l_prefix.len() {
            // l is a prefix of r - shorten r_prefix
            let r_prefix = &r_prefix[n..];
            // if l has a value but not r, we found one
            // if one or more of lc are not a subset of r, we are done
            !l.value.is_some()
                && l.children()
                    .iter()
                    .all(|lc| Self::is_subset0(lc, lc.prefix(), r, r_prefix))
        } else if n == r_prefix.len() {
            // r is a prefix of l - shorten L_prefix
            let l_prefix = &l_prefix[n..];
            // if l is a subset of none of rc, we are done
            r.children()
                .iter()
                .any(|rc| Self::is_subset0(l, l_prefix, rc, rc.prefix()))
        } else {
            // disjoint
            false
        }
    }

    pub fn intersects<W: Debug>(&self, that: &RadixTree<K, W>) -> bool {
        Self::intersects0(self, self.prefix(), that, that.prefix())
    }

    fn intersects0<W: Debug>(
        l: &Self,
        l_prefix: &[K],
        r: &RadixTree<K, W>,
        r_prefix: &[K],
    ) -> bool {
        let n = common_prefix(&l_prefix, &r_prefix);
        if n == l_prefix.len() && n == r_prefix.len() {
            // prefixes are identical
            (l.value().is_some() && r.value().is_some())
                || BoolOpMergeState::merge(l.children(), r.children(), IntersectOp)
        } else if n == l_prefix.len() {
            // l is a prefix of r
            let r_prefix = &r_prefix[n..];
            l.children()
                .iter()
                .any(|lc| Self::intersects0(lc, lc.prefix(), r, r_prefix))
        } else if n == r_prefix.len() {
            // r is a prefix of l
            let l_prefix = &l_prefix[n..];
            r.children()
                .iter()
                .any(|rc| Self::intersects0(l, l_prefix, rc, rc.prefix()))
        } else {
            // disjoint
            false
        }
    }
}

impl<K: Ord + Copy + Debug, V: Debug + Clone> RadixTree<K, V> {
    pub fn leaf(value: V) -> Self {
        Self {
            prefix: Fragment::default(),
            value: Some(value),
            children: Default::default(),
        }
    }

    pub fn single(key: &[K], value: V) -> Self {
        Self {
            prefix: key.into(),
            value: Some(value),
            children: Vec::new(),
        }
    }

    pub fn prepend(&mut self, prefix: &[K]) {
        if !prefix.is_empty() {
            let mut prefix1 = SmallVec::new();
            prefix1.extend_from_slice(prefix);
            prefix1.extend_from_slice(self.prefix());
            self.prefix = prefix1.into();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value().is_none() && self.children().is_empty()
    }

    /// create an artificial split at offset n
    /// splitting at n >= prefix.len() is an error
    fn split(&mut self, n: usize) {
        assert!(n < self.prefix().len());
        let first = self.prefix()[..n].into();
        let rest = self.prefix()[n..].into();
        let mut split = Self {
            prefix: first,
            value: None,
            children: Vec::new(),
        };
        std::mem::swap(self, &mut split);
        let mut child = split;
        child.prefix = rest;
        self.children.push(child);
    }

    /// removes degenerate node again
    fn unsplit(&mut self) {
        // a single child and no own value is degenerate
        if self.children.len() == 1 && self.value.is_none() {
            let mut child = self.children.pop().unwrap();
            child.prepend(&self.prefix);
            *self = child;
        }
    }

    fn clone_shortened(&self, n: usize) -> Self {
        assert!(n < self.prefix().len());
        let mut res = self.clone();
        res.prefix = res.prefix()[n..].into();
        res
    }

    pub fn union_with(&mut self, that: &RadixTree<K, V>) {
        self.outer_combine_with(that, |l, r| {
            if l.is_none() {
                if r.is_some() {
                    *l = r.clone();
                }
            }
        })
    }

    pub fn intersection_with(&mut self, that: &RadixTree<K, V>) {
        self.inner_combine_with(that, |_, _| true)
    }

    pub fn difference_with(&mut self, that: &RadixTree<K, V>) {
        self.left_combine_with(that, |_, _| false)
    }

    fn outer_combine_with(&mut self, that: &Self, f: impl Fn(&mut Option<V>, &Option<V>) + Copy) {
        let n = common_prefix(self.prefix(), that.prefix());
        if n == self.prefix().len() && n == that.prefix().len() {
            // prefixes are identical
            f(&mut self.value, that.value());
            self.outer_combine_children_with(that.children(), f);
        } else if n == self.prefix().len() {
            // self is a prefix of that
            let that = that.clone_shortened(n);
            self.outer_combine_children_with(&[that], f);
        } else if n == that.prefix().len() {
            // that is a prefix of self
            // split at the offset, then merge in that
            // we must not swap sides!
            self.split(n);
            // self now has the same prefix as that, so just repeat the code
            // from where prefixes are identical
            f(&mut self.value, that.value());
            self.outer_combine_children_with(that.children(), f);
            self.unsplit();
        } else {
            assert!(n > 0);
            // disjoint
            self.split(n);
            self.children.push(that.clone_shortened(n));
            self.children.sort_by_key(|x| x.prefix()[0]);
            self.unsplit();
        }
    }

    fn outer_combine_children_with(
        &mut self,
        rhs: &[Self],
        f: impl Fn(&mut Option<V>, &Option<V>) + Copy,
    ) {
        // this convoluted stuff is because we don't have an InPlaceMergeStateRef for Vec
        // so we convert into a smallvec, perform the ops there, then convert back.
        let mut tmp = Vec::new();
        std::mem::swap(&mut self.children, &mut tmp);
        let mut t = SmallVec::<[Self; 0]>::from_vec(tmp);
        InPlaceMergeStateRef::merge(&mut t, &rhs, OuterCombineOp(f));
        self.children = t.into_vec()
    }

    fn inner_combine_with(
        &mut self,
        that: &RadixTree<K, V>,
        f: impl Fn(&mut V, &V) -> bool + Copy,
    ) {
        let n = common_prefix(self.prefix(), that.prefix());
        if n == self.prefix().len() && n == that.prefix().len() {
            // prefixes are identical
            if let (Some(v), Some(w)) = (&mut self.value, that.value()) {
                if !f(v, w) {
                    self.value = None;
                }
            } else {
                self.value = None;
            }
            self.inner_combine_children_with(that.children(), f);
        } else if n == self.prefix().len() {
            // self is a prefix of that
            self.value = None;
            let that = that.clone_shortened(n);
            self.inner_combine_children_with(&[that], f);
        } else if n == that.prefix().len() {
            // that is a prefix of self
            // split at the offset, then merge in that
            // we must not swap sides!
            self.split(n);
            self.inner_combine_children_with(that.children(), f);
            self.unsplit();
        } else {
            // disjoint
            self.value = None;
            self.children.clear();
        }
    }

    fn inner_combine_children_with(
        &mut self,
        rhs: &[RadixTree<K, V>],
        f: impl Fn(&mut V, &V) -> bool + Copy,
    ) {
        // this convoluted stuff is because we don't have an InPlaceMergeStateRef for Vec
        // so we convert into a smallvec, perform the ops there, then convert back.
        let mut tmp = Vec::new();
        std::mem::swap(&mut self.children, &mut tmp);
        let mut t = SmallVec::<[Self; 0]>::from_vec(tmp);
        InPlaceMergeStateRef::merge(&mut t, &rhs, InnerCombineOp(f));
        self.children = t.into_vec()
    }

    fn left_combine_with(&mut self, that: &RadixTree<K, V>, f: impl Fn(&mut V, &V) -> bool + Copy) {
        let n = common_prefix(self.prefix(), that.prefix());
        if n == self.prefix().len() && n == that.prefix().len() {
            // prefixes are identical
            if let (Some(v), Some(w)) = (&mut self.value, that.value()) {
                if !f(v, w) {
                    self.value = None;
                }
            } else {
                self.value = None;
            }
            self.left_combine_children_with(that.children(), f);
        } else if n == self.prefix().len() {
            // self is a prefix of that
            let that = that.clone_shortened(n);
            self.left_combine_children_with(&[that], f);
        } else if n == that.prefix().len() {
            // that is a prefix of self
            self.split(n);
            self.left_combine_children_with(that.children(), f);
            self.unsplit();
        } else {
            // disjoint
            self.value = None;
            self.children.clear();
        }
    }

    fn left_combine_children_with(
        &mut self,
        rhs: &[RadixTree<K, V>],
        f: impl Fn(&mut V, &V) -> bool + Copy,
    ) {
        // this convoluted stuff is because we don't have an InPlaceMergeStateRef for Vec
        // so we convert into a smallvec, perform the ops there, then convert back.
        let mut tmp = Vec::new();
        std::mem::swap(&mut self.children, &mut tmp);
        let mut t = SmallVec::<[Self; 0]>::from_vec(tmp);
        InPlaceMergeStateRef::merge(&mut t, &rhs, LeftCombineOp(f));
        self.children = t.into_vec()
    }
}

struct IntersectOp;

impl<'a, K, V, W, I> MergeOperation<I> for IntersectOp
where
    K: Ord + Copy + Debug,
    I: MergeStateMut<A = RadixTree<K, V>, B = RadixTree<K, W>>,
    V: Debug,
    W: Debug,
{
    fn cmp(&self, a: &RadixTree<K, V>, b: &RadixTree<K, W>) -> Ordering {
        a.prefix()[0].cmp(&b.prefix()[0])
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, false)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, false)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        let a = &m.a_slice()[0];
        let b = &m.b_slice()[0];
        // if this is true, we have found an intersection and can abort.
        let take = a.intersects(b);
        m.advance_a(1, take)?;
        m.advance_b(1, false)
    }
}
struct NonSubsetOp;

impl<'a, K, V, W, I> MergeOperation<I> for NonSubsetOp
where
    K: Ord + Copy + Debug,
    I: MergeStateMut<A = RadixTree<K, V>, B = RadixTree<K, W>>,
    V: Debug,
    W: Debug,
{
    fn cmp(&self, a: &RadixTree<K, V>, b: &RadixTree<K, W>) -> Ordering {
        a.prefix()[0].cmp(&b.prefix()[0])
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, false)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        let a = &m.a_slice()[0];
        let b = &m.b_slice()[0];
        // if this is true, we have found a value of a that is not in b, and we can abort
        let take = !a.is_subset(b);
        m.advance_a(1, take)?;
        m.advance_b(1, false)
    }
}

/// In place merge operation
struct OuterCombineOp<F>(F);

impl<'a, F, K, V, I> MergeOperation<I> for OuterCombineOp<F>
where
    F: Fn(&mut Option<V>, &Option<V>) + Copy,
    V: Debug + Clone,
    K: Ord + Copy + Debug,
    I: MutateInput<A = RadixTree<K, V>, B = RadixTree<K, V>>,
{
    fn cmp(&self, a: &RadixTree<K, V>, b: &RadixTree<K, V>) -> Ordering {
        a.prefix()[0].cmp(&b.prefix()[0])
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, true)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        let (a, b) = m.source_slices_mut();
        let av = &mut a[0];
        let bv = &b[0];
        av.outer_combine_with(bv, self.0);
        // we have modified av in place. We are only going to take it over if it
        // is non-empty, otherwise we skip it.
        let take = !av.is_empty();
        m.advance_a(1, take)?;
        m.advance_b(1, false)
    }
}

/// In place intersection operation
struct InnerCombineOp<F>(F);

impl<'a, K, V, F, I> MergeOperation<I> for InnerCombineOp<F>
where
    K: Ord + Copy + Debug,
    V: Debug + Clone,
    F: Fn(&mut V, &V) -> bool + Copy,
    I: MutateInput<A = RadixTree<K, V>, B = RadixTree<K, V>>,
{
    fn cmp(&self, a: &RadixTree<K, V>, b: &RadixTree<K, V>) -> Ordering {
        a.prefix()[0].cmp(&b.prefix()[0])
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, false)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, false)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        let (a, b) = m.source_slices_mut();
        let av = &mut a[0];
        let bv = &b[0];
        av.inner_combine_with(bv, self.0);
        // we have modified av in place. We are only going to take it over if it
        // is non-empty, otherwise we skip it.
        let take = !av.is_empty();
        m.advance_a(1, take)?;
        m.advance_b(1, false)
    }
}

/// In place intersection operation
struct LeftCombineOp<F>(F);

impl<'a, K, V, F, I> MergeOperation<I> for LeftCombineOp<F>
where
    K: Ord + Copy + Debug,
    V: Debug + Clone,
    F: Fn(&mut V, &V) -> bool + Copy,
    I: MutateInput<A = RadixTree<K, V>, B = RadixTree<K, V>>,
{
    fn cmp(&self, a: &RadixTree<K, V>, b: &RadixTree<K, V>) -> Ordering {
        a.prefix()[0].cmp(&b.prefix()[0])
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, false)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        let (a, b) = m.source_slices_mut();
        let av = &mut a[0];
        let bv = &b[0];
        av.left_combine_with(bv, self.0);
        // we have modified av in place. We are only going to take it over if it
        // is non-empty, otherwise we skip it.
        let take = !av.is_empty();
        m.advance_a(1, take)?;
        m.advance_b(1, false)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn smoke_test() {
        let mut res = RadixTree::default();
        let keys = ["aabbcc", "aabb", "aabbee"];
        let nope = ["xaabbcc", "aabbx", "aabbx", "aabbeex"];
        for key in keys {
            let x = RadixTree::single(key.as_bytes(), ());
            res.union_with(&x);
            // any set is subset of itself
            assert!(res.is_subset(&res));
        }
        for key in nope {
            assert!(!res.contains_key(key.as_bytes()));
            // keys not contained in the set must not be a subset
            let mut t = RadixTree::single(key.as_bytes(), ());
            assert!(!t.is_subset(&res));
            t.intersection_with(&res);
            assert!(t.is_empty());
        }
        for key in keys {
            assert!(res.contains_key(key.as_bytes()));
            // keys contained in the set must be a subset
            assert!(RadixTree::single(key.as_bytes(), ()).is_subset(&res));
            let mut t = RadixTree::single(key.as_bytes(), ());
            assert!(t.is_subset(&res));
            t.intersection_with(&res);
            assert!(t == RadixTree::single(key.as_bytes(), ()));
        }
    }
}
