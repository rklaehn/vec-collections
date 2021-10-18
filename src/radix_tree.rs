use std::{cmp::Ordering, fmt::Debug, iter::FromIterator, sync::Arc};

use smallvec::SmallVec;

use crate::{
    binary_merge::{EarlyOut, MergeOperation},
    merge_state::{
        BoolOpMergeState, CloneConverter, InPlaceMergeStateRef, MergeStateMut, MutateInput,
        NoConverter,
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

trait AbstractRadixTree<K, V>: Sized {
    fn prefix(&self) -> &[K];
    fn value(&self) -> &Option<V>;
    fn children(&self) -> &[Self];

    fn is_empty(&self) -> bool {
        self.value().is_none() && self.children().is_empty()
    }
}

impl<E: Ord + Copy + Debug, K: AsRef<[E]>, V: Debug + Clone> FromIterator<(K, V)>
    for RadixTree<E, V>
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut res = RadixTree::default();
        for (k, v) in iter.into_iter() {
            res.union_with(&RadixTree::single(k.as_ref(), v))
        }
        res
    }
}

#[derive(Debug, Clone)]
pub struct IterKey<K>(Arc<Vec<K>>);

impl<K: Clone> IterKey<K> {
    fn new() -> Self {
        Self(Arc::new(Vec::new()))
    }

    fn append(&mut self, data: &[K]) {
        let elems = Arc::make_mut(&mut self.0);
        elems.extend_from_slice(data);
    }

    fn pop(&mut self, n: usize) {
        let elems = Arc::make_mut(&mut self.0);
        elems.truncate(elems.len().saturating_sub(n));
    }
}

impl<T> AsRef<[T]> for IterKey<T> {
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

pub struct Iter<'a, K, V> {
    path: IterKey<K>,
    stack: Vec<(&'a RadixTree<K, V>, usize)>,
}

impl<'a, K: Clone, V> Iter<'a, K, V> {
    fn new(tree: &'a RadixTree<K, V>) -> Self {
        Self {
            stack: vec![(tree, 0)],
            path: IterKey::new(),
        }
    }

    fn tree(&self) -> &'a RadixTree<K, V> {
        self.stack.last().unwrap().0
    }

    fn inc(&mut self) -> Option<usize> {
        let pos = &mut self.stack.last_mut().unwrap().1;
        let res = if *pos == 0 { None } else { Some(*pos - 1) };
        *pos += 1;
        res
    }
}

impl<'a, K: Ord + Copy + Debug, V: Debug> Iterator for Iter<'a, K, V> {
    type Item = (IterKey<K>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while !self.stack.is_empty() {
            if let Some(pos) = self.inc() {
                if pos < self.tree().children.len() {
                    let child = &self.tree().children[pos];
                    self.path.append(child.prefix());
                    self.stack.push((child, 0));
                } else {
                    self.path.pop(self.tree().prefix().len());
                    self.stack.pop();
                }
            } else {
                if let Some(value) = self.tree().value.as_ref() {
                    return Some((self.path.clone(), value));
                }
            }
        }
        None
    }
}

pub struct Values<'a, K, V> {
    stack: Vec<(&'a RadixTree<K, V>, usize)>,
}

impl<'a, K, V> Values<'a, K, V> {
    fn new(tree: &'a RadixTree<K, V>) -> Self {
        Self {
            stack: vec![(tree, 0)],
        }
    }

    fn tree(&self) -> &'a RadixTree<K, V> {
        self.stack.last().unwrap().0
    }

    fn inc(&mut self) -> Option<usize> {
        let pos = &mut self.stack.last_mut().unwrap().1;
        let res = if *pos == 0 { None } else { Some(*pos - 1) };
        *pos += 1;
        res
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.stack.is_empty() {
            if let Some(pos) = self.inc() {
                if pos < self.tree().children.len() {
                    self.stack.push((&self.tree().children[pos], 0));
                } else {
                    self.stack.pop();
                }
            } else {
                if let Some(value) = self.tree().value.as_ref() {
                    return Some(value);
                }
            }
        }
        None
    }
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
    pub fn values(&self) -> Values<'_, K, V> {
        Values::new(self)
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter::new(self)
    }

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
        let n = common_prefix(l_prefix, r_prefix);
        if n == l_prefix.len() && n == r_prefix.len() {
            // prefixes are identical
            (!l.value().is_some() || r.value().is_some())
                && !BoolOpMergeState::merge(l.children(), r.children(), NonSubsetOp)
        } else if n == l_prefix.len() {
            // l is a prefix of r - shorten r_prefix
            let r_prefix = &r_prefix[n..];
            // if l has a value but not r, we found one
            // if one or more of lc are not a subset of r, we are done
            l.value.is_none()
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

    pub fn is_disjoint<W: Debug>(&self, that: &RadixTree<K, W>) -> bool {
        !self.intersects(that)
    }

    fn intersects0<W: Debug>(
        l: &Self,
        l_prefix: &[K],
        r: &RadixTree<K, W>,
        r_prefix: &[K],
    ) -> bool {
        let n = common_prefix(l_prefix, r_prefix);
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
        Self {
            prefix: self.prefix()[n..].into(),
            value: self.value.clone(),
            children: self.children.clone(),
        }
    }

    /// Left biased union with another tree of the same key and value type
    pub fn union_with(&mut self, that: &RadixTree<K, V>) {
        self.outer_combine_with(that, |_, _| true)
    }

    /// Intersection with another tree of the same key type
    pub fn intersection_with<W: Clone + Debug>(&mut self, that: &RadixTree<K, W>) {
        self.inner_combine_with(that, |_, _| true)
    }

    /// Difference with another tree of the same key type
    pub fn difference_with<W: Clone + Debug>(&mut self, that: &RadixTree<K, W>) {
        self.left_combine_with(that, |_, _| false)
    }

    /// outer combine of `self` tree with `that` tree
    ///
    /// outer means that elements that are in `self` but not in `that` or vice versa are copied.
    /// for elements that are in both trees, it is possible to customize how they are combined.
    /// `f` can mutate the value of `self` in place, or return false to remove the value.
    fn outer_combine_with(&mut self, that: &Self, f: impl Fn(&mut V, &V) -> bool + Copy) {
        let n = common_prefix(self.prefix(), that.prefix());
        if n == self.prefix().len() && n == that.prefix().len() {
            // prefixes are identical
            if let Some(w) = that.value() {
                if let Some(v) = &mut self.value {
                    if !f(v, w) {
                        self.value = None;
                    }
                } else {
                    self.value = Some(w.clone())
                }
            }
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
            if let Some(w) = that.value() {
                if let Some(v) = &mut self.value {
                    if !f(v, w) {
                        self.value = None;
                    }
                } else {
                    self.value = Some(w.clone())
                }
            }
            self.outer_combine_children_with(that.children(), f);
        } else {
            // disjoint
            self.split(n);
            self.children.push(that.clone_shortened(n));
            self.children.sort_by_key(|x| x.prefix()[0]);
        }
        self.unsplit();
    }

    fn outer_combine_children_with(&mut self, rhs: &[Self], f: impl Fn(&mut V, &V) -> bool + Copy) {
        // this convoluted stuff is because we don't have an InPlaceMergeStateRef for Vec
        // so we convert into a smallvec, perform the ops there, then convert back.
        let mut tmp = Vec::new();
        std::mem::swap(&mut self.children, &mut tmp);
        let mut t = SmallVec::<[Self; 0]>::from_vec(tmp);
        InPlaceMergeStateRef::merge(&mut t, &rhs, OuterCombineOp(f), CloneConverter);
        self.children = t.into_vec()
    }

    /// inner combine of `self` tree with `that` tree
    ///
    /// inner means that elements that are in `self` but not in `that` or vice versa are removed.
    /// for elements that are in both trees, it is possible to customize how they are combined.
    /// `f` can mutate the value of `self` in place, or return false to remove the value.
    fn inner_combine_with<W: Clone + Debug>(
        &mut self,
        that: &RadixTree<K, W>,
        f: impl Fn(&mut V, &W) -> bool + Copy,
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
        } else {
            // disjoint
            self.value = None;
            self.children.clear();
        }
        self.unsplit();
    }

    fn inner_combine_children_with<W: Clone + Debug>(
        &mut self,
        rhs: &[RadixTree<K, W>],
        f: impl Fn(&mut V, &W) -> bool + Copy,
    ) {
        // this convoluted stuff is because we don't have an InPlaceMergeStateRef for Vec
        // so we convert into a smallvec, perform the ops there, then convert back.
        let mut tmp = Vec::new();
        std::mem::swap(&mut self.children, &mut tmp);
        let mut t = SmallVec::<[Self; 0]>::from_vec(tmp);
        InPlaceMergeStateRef::merge(&mut t, &rhs, InnerCombineOp(f), NoConverter);
        self.children = t.into_vec()
    }

    /// Left combine of `self` tree with `that` tree
    ///
    /// Left means that elements that are in `self` but not in `that` are kept, but elements that
    /// are in `that` but not in `self` are dropped.
    ///
    /// For elements that are in both trees, it is possible to customize how they are combined.
    /// `f` can mutate the value of `self` in place, or return false to remove the value.
    fn left_combine_with<W: Debug + Clone>(
        &mut self,
        that: &RadixTree<K, W>,
        f: impl Fn(&mut V, &W) -> bool + Copy,
    ) {
        let n = common_prefix(self.prefix(), that.prefix());
        if n == self.prefix().len() && n == that.prefix().len() {
            // prefixes are identical
            if let Some(w) = that.value() {
                if let Some(v) = &mut self.value {
                    if !f(v, w) {
                        self.value = None;
                    }
                }
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
        } else {
            // disjoint, nothing to do
        }
        self.unsplit();
    }

    fn left_combine_children_with<W: Debug + Clone>(
        &mut self,
        rhs: &[RadixTree<K, W>],
        f: impl Fn(&mut V, &W) -> bool + Copy,
    ) {
        // this convoluted stuff is because we don't have an InPlaceMergeStateRef for Vec
        // so we convert into a smallvec, perform the ops there, then convert back.
        let mut tmp = Vec::new();
        std::mem::swap(&mut self.children, &mut tmp);
        let mut t = SmallVec::<[Self; 0]>::from_vec(tmp);
        InPlaceMergeStateRef::merge(&mut t, &rhs, LeftCombineOp(f), NoConverter);
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
    F: Fn(&mut V, &V) -> bool + Copy,
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

impl<'a, K, V, W, F, I> MergeOperation<I> for InnerCombineOp<F>
where
    K: Ord + Copy + Debug,
    V: Debug + Clone,
    W: Debug + Clone,
    F: Fn(&mut V, &W) -> bool + Copy,
    I: MutateInput<A = RadixTree<K, V>, B = RadixTree<K, W>>,
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

impl<'a, K, V, W, F, I> MergeOperation<I> for LeftCombineOp<F>
where
    K: Ord + Copy + Debug,
    V: Debug + Clone,
    W: Debug + Clone,
    F: Fn(&mut V, &W) -> bool + Copy,
    I: MutateInput<A = RadixTree<K, V>, B = RadixTree<K, W>>,
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
mod test {
    use std::collections::BTreeSet;

    use super::*;
    use crate::obey::*;
    use quickcheck::*;

    impl Arbitrary for RadixTree<u8, ()> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let t: Vec<String> = Arbitrary::arbitrary(g);
            t.iter().map(|x| (x.as_bytes().to_vec(), ())).collect()
        }
    }

    impl TestSamples<Vec<u8>, bool> for RadixTree<u8, ()> {
        fn samples(&self, res: &mut BTreeSet<Vec<u8>>) {
            res.insert(Vec::new());
        }

        fn at(&self, elem: Vec<u8>) -> bool {
            self.contains_key(&elem)
        }
    }

    type Test = RadixTree<u8, ()>;
    type Reference = BTreeSet<Vec<u8>>;

    fn r2t(r: &Reference) -> Test {
        r.iter().map(|t| (t.to_vec(), ())).collect()
    }

    quickcheck! {

        fn is_disjoint_sample(a: Test, b: Test) -> bool {
            binary_property_test(&a, &b, a.is_disjoint(&b), |a, b| !(a & b))
        }

        fn is_subset_sample(a: Test, b: Test) -> bool {
            binary_property_test(&a, &b, a.is_subset(&b), |a, b| !a | b)
        }

        fn union_sample(a: Test, b: Test) -> bool {
            let mut r = a.clone();
            r.union_with(&b);
            binary_element_test(&a, &b, r, |a, b| a | b)
        }

        fn intersection_sample(a: Test, b: Test) -> bool {
            let mut r = a.clone();
            r.intersection_with(&b);
            binary_element_test(&a, &b, r, |a, b| a & b)
        }

        // fn xor_sample(a: Test, b: Test) -> bool {
        //     binary_element_test(&a, &b, &a ^ &b, |a, b| a ^ b)
        // }

        fn diff_sample(a: Test, b: Test) -> bool {
            let mut r = a.clone();
            r.difference_with(&b);
            binary_element_test(&a, &b, r, |a, b| a & !b)
        }

        fn union(a: Reference, b: Reference) -> bool {
            let a1: Test = r2t(&a);
            let b1: Test = r2t(&b);
            let mut r1 = a1;
            r1.union_with(&b1);
            let expected = r2t(&a.union(&b).cloned().collect());
            expected == r1
        }

        fn intersection(a: Reference, b: Reference) -> bool {
            let a1: Test = r2t(&a);
            let b1: Test = r2t(&b);
            let mut r1 = a1;
            r1.intersection_with(&b1);
            let expected = r2t(&a.intersection(&b).cloned().collect());
            expected == r1
        }

        fn difference(a: Reference, b: Reference) -> bool {
            let a = a.into_iter().take(2).collect();
            let b = b.into_iter().take(2).collect();
            let a1: Test = r2t(&a);
            let b1: Test = r2t(&b);
            let mut r1 = a1;
            r1.difference_with(&b1);
            let expected = r2t(&a.difference(&b).cloned().collect());
            if expected != r1 {
                println!("expected:{:#?}\nvalue:{:#?}", expected, r1);
            }
            expected == r1
        }

        fn is_disjoint(a: Reference, b: Reference) -> bool {
            let a1: Test = r2t(&a);
            let b1: Test = r2t(&b);
            let actual = a1.is_disjoint(&b1);
            let expected = a.is_disjoint(&b);
            expected == actual
        }

        fn is_subset(a: Reference, b: Reference) -> bool {
            let a1: Test = r2t(&a);
            let b1: Test = r2t(&b);
            let actual = a1.is_subset(&b1);
            let expected = a.is_subset(&b);
            expected == actual
        }

        fn contains(a: Reference, b: Vec<u8>) -> bool {
            let a1: Test = r2t(&a);
            let expected = a.contains(&b);
            let actual = a1.contains_key(&b);
            expected == actual
        }
    }

    // bitop_assign_consistent!(Test);
    // set_predicate_consistent!(Test);
    // bitop_symmetry!(Test);
    // bitop_empty!(Test);

    #[test]
    fn values_iter() {
        let elems = &["abc", "ab", "a", "ba"];
        let tree = elems
            .iter()
            .map(|x| (x.as_bytes(), (*x).to_owned()))
            .collect::<RadixTree<_, _>>();
        for x in tree.values() {
            println!("{}", x);
        }
        for (k, v) in tree.iter() {
            println!("{:?} {}", k, v);
        }
    }

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
            let mut dif = res.clone();
            dif.difference_with(&RadixTree::single(key.as_bytes(), ()));
            assert_eq!(dif, res);
        }
        for key in keys {
            assert!(res.contains_key(key.as_bytes()));
            // keys contained in the set must be a subset
            assert!(RadixTree::single(key.as_bytes(), ()).is_subset(&res));
            let mut t = RadixTree::single(key.as_bytes(), ());
            assert!(t.is_subset(&res));
            t.intersection_with(&res);
            assert!(t == RadixTree::single(key.as_bytes(), ()));
            let mut dif = res.clone();
            dif.difference_with(&RadixTree::single(key.as_bytes(), ()));
            assert!(!dif.contains_key(key.as_bytes()));
        }
    }

    fn differencex(a: Reference, b: Reference) -> bool {
        let a = a.into_iter().take(2).collect();
        let b = b.into_iter().take(2).collect();
        let a1: Test = r2t(&a);
        let b1: Test = r2t(&b);
        let mut r1 = a1;
        r1.difference_with(&b1);
        let expected = r2t(&a.difference(&b).cloned().collect());
        if expected != r1 {
            println!("expected:{:#?}\nvalue:{:#?}", expected, r1);
        }
        expected == r1
    }

    #[test]
    fn difference1() {
        use maplit::btreeset;
        let a = btreeset! {vec![1,0]};
        let b = btreeset! {vec![1,1]};
        println!("{:#?}", r2t(&a));
        println!("{:#?}", r2t(&b));
        assert!(differencex(a, b));
    }
}
