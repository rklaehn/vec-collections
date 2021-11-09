use std::{borrow::Borrow, cmp::Ordering, fmt::Debug, marker::PhantomData, ops::Deref, sync::Arc};

pub trait TKey: Debug + Ord + Copy + Archive<Archived = Self> + Send + Sync + 'static {}

impl<T: Debug + Ord + Copy + Archive<Archived = T> + Send + Sync + 'static> TKey for T {}

pub trait TValue: Debug + Clone + Archive<Archived = Self> + Send + Sync + 'static {}

impl<T: Debug + Clone + Archive<Archived = T> + Send + Sync + 'static> TValue for T {}

use rkyv::Archive;
#[cfg(feature = "rkyv")]
mod lazy_radix_tree;
#[cfg(feature = "rkyv")]
pub use lazy_radix_tree::*;
#[cfg(feature = "rkyv")]
mod arc_radix_tree;
#[cfg(feature = "rkyv")]
pub use arc_radix_tree::*;
use smallvec::{Array, SmallVec};
use sorted_iter::sorted_pair_iterator::SortedByKey;
mod radix_tree;
use crate::{
    binary_merge::{EarlyOut, MergeOperation},
    merge_state::{
        BoolOpMergeState, Converter, InPlaceMergeStateRef, MergeStateMut, MutateInput, NoConverter,
        VecMergeState,
    },
};
pub use radix_tree::*;

/// A path fragment
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment<T>(SmallVec<[T; 16]>);

impl<T> AsRef<[T]> for Fragment<T> {
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T> Borrow<[T]> for Fragment<T> {
    fn borrow(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T> Deref for Fragment<T> {
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

// common prefix of two slices.
fn common_prefix<'a, T: Eq>(a: &'a [T], b: &'a [T]) -> usize {
    a.iter().zip(b).take_while(|(a, b)| a == b).count()
}

pub trait AbstractRadixTreeMut<K: TKey, V: TValue>:
    AbstractRadixTree<K, V, Materialized = Self> + Clone + Default
{
    fn new(prefix: Fragment<K>, value: Option<V>, children: Vec<Self>) -> Self;
    fn value_mut(&mut self) -> &mut Option<V>;
    fn children_mut(&mut self) -> &mut Vec<Self>;
    fn prefix_mut(&mut self) -> &mut Fragment<K>;

    fn empty() -> Self {
        Self::default()
    }

    fn leaf(value: V) -> Self {
        Self::new(Fragment::default(), Some(value), Default::default())
    }

    fn single(key: &[K], value: V) -> Self {
        Self::new(key.into(), Some(value), Vec::new())
    }

    fn prepend(&mut self, prefix: &[K]) {
        if !prefix.is_empty() && !self.is_empty() {
            let mut prefix1 = SmallVec::new();
            prefix1.extend_from_slice(prefix);
            prefix1.extend_from_slice(self.prefix());
            *self.prefix_mut() = prefix1.into();
        }
    }

    fn filter_prefix(&self, prefix: &[K]) -> Self {
        match find(self, prefix) {
            FindResult::Found(tree) => {
                let mut res = tree.clone();
                *res.prefix_mut() = prefix.into();
                res
            }
            FindResult::Prefix { tree, rt } => {
                let mut res = tree.clone();
                let p = res.prefix();
                *res.prefix_mut() = Fragment::from(&p[p.len() - rt..]);
                res.prepend(prefix);
                res
            }
            FindResult::NotFound { .. } => Self::default(),
        }
    }

    /// create an artificial split at offset n
    /// splitting at n >= prefix.len() is an error
    fn split(&mut self, n: usize) {
        assert!(n < self.prefix().len());
        let first = self.prefix()[..n].into();
        let rest = self.prefix()[n..].into();
        let mut split = Self::new(first, None, Vec::new());
        std::mem::swap(self, &mut split);
        let mut child = split;
        *child.prefix_mut() = rest;
        self.children_mut().push(child);
    }

    /// removes degenerate node again
    fn unsplit(&mut self) {
        // remove all empty children
        // this might sometimes not be necessary, but it is tricky to find out when.
        self.children_mut().retain(|x| !x.is_empty());
        // a single child and no own value is degenerate
        if self.children().len() == 1 && self.value().is_none() {
            let mut child = self.children_mut().pop().unwrap();
            child.prepend(self.prefix());
            *self = child;
        }
        // canonicalize prefix for empty node
        // this might sometimes not be necessary, but it is tricky to find out when.
        if self.is_empty() {
            *self.prefix_mut() = Fragment::default();
        }
    }

    /// Left biased union with another tree of the same key and value type
    fn union_with(
        &mut self,
        that: &impl AbstractRadixTree<K, V, Materialized = Self::Materialized>,
    ) {
        self.outer_combine_with(that, |_, _| true)
    }

    /// Intersection with another tree of the same key type
    fn intersection_with<W: TValue>(&mut self, that: &impl AbstractRadixTree<K, W>) {
        self.inner_combine_with(that, |_, _| true)
    }

    /// Difference with another tree of the same key type
    fn difference_with<W: TValue>(&mut self, that: &impl AbstractRadixTree<K, W>) {
        self.left_combine_with(that, |_, _| false)
    }

    /// outer combine of `self` tree with `that` tree
    ///
    /// outer means that elements that are in `self` but not in `that` or vice versa are copied.
    /// for elements that are in both trees, it is possible to customize how they are combined.
    /// `f` can mutate the value of `self` in place, or return false to remove the value.
    fn outer_combine_with(
        &mut self,
        that: &impl AbstractRadixTree<K, V, Materialized = Self::Materialized>,
        f: impl Fn(&mut V, &V) -> bool + Copy,
    ) {
        let n = common_prefix(self.prefix(), that.prefix());
        if n == self.prefix().len() && n == that.prefix().len() {
            // prefixes are identical
            if let Some(w) = that.value() {
                if let Some(v) = &mut self.value_mut() {
                    if !f(v, w) {
                        *self.value_mut() = None;
                    }
                } else {
                    *self.value_mut() = Some(w.clone())
                }
            }
            self.outer_combine_children_with(that.children(), f);
        } else if n == self.prefix().len() {
            // self is a prefix of that
            let that = that.materialize_shortened(n);
            self.outer_combine_children_with(&[that], f);
        } else if n == that.prefix().len() {
            // that is a prefix of self
            // split at the offset, then merge in that
            // we must not swap sides!
            self.split(n);
            // self now has the same prefix as that, so just repeat the code
            // from where prefixes are identical
            if let Some(w) = that.value() {
                if let Some(v) = &mut self.value_mut() {
                    if !f(v, w) {
                        *self.value_mut() = None;
                    }
                } else {
                    *self.value_mut() = Some(w.clone())
                }
            }
            self.outer_combine_children_with(that.children(), f);
        } else {
            // disjoint
            self.split(n);
            self.children_mut().push(that.materialize_shortened(n));
            self.children_mut().sort_by_key(|x| x.prefix()[0]);
        }
        self.unsplit();
    }

    fn outer_combine_children_with(
        &mut self,
        rhs: &[impl AbstractRadixTree<K, V, Materialized = Self::Materialized>],
        f: impl Fn(&mut V, &V) -> bool + Copy,
    ) {
        // this convoluted stuff is because we don't have an InPlaceMergeStateRef for Vec
        // so we convert into a smallvec, perform the ops there, then convert back.
        let mut tmp = Vec::new();
        std::mem::swap(self.children_mut(), &mut tmp);
        let mut t = SmallVec::<[Self; 0]>::from_vec(tmp);
        InPlaceMergeStateRef::merge(
            &mut t,
            &rhs,
            OuterCombineOp(f, PhantomData),
            RadixTreeConverter(PhantomData),
        );
        *self.children_mut() = t.into_vec()
    }

    /// inner combine of `self` tree with `that` tree
    ///
    /// inner means that elements that are in `self` but not in `that` or vice versa are removed.
    /// for elements that are in both trees, it is possible to customize how they are combined.
    /// `f` can mutate the value of `self` in place, or return false to remove the value.
    fn inner_combine_with<W: TValue>(
        &mut self,
        that: &impl AbstractRadixTree<K, W>,
        f: impl Fn(&mut V, &W) -> bool + Copy,
    ) {
        let n = common_prefix(self.prefix(), that.prefix());
        if n == self.prefix().len() && n == that.prefix().len() {
            // prefixes are identical
            if let (Some(v), Some(w)) = (self.value_mut(), that.value()) {
                if !f(v, w) {
                    *self.value_mut() = None;
                }
            } else {
                *self.value_mut() = None;
            }
            self.inner_combine_children_with(that.children(), f);
        } else if n == self.prefix().len() {
            // self is a prefix of that
            *self.value_mut() = None;
            let that = that.materialize_shortened(n);
            self.inner_combine_children_with(&[that], f);
        } else if n == that.prefix().len() {
            // that is a prefix of self
            // split at the offset, then merge in that
            // we must not swap sides!
            self.split(n);
            self.inner_combine_children_with(that.children(), f);
        } else {
            // disjoint
            *self.value_mut() = None;
            self.children_mut().clear();
        }
        self.unsplit();
    }

    fn inner_combine_children_with<W: TValue>(
        &mut self,
        rhs: &[impl AbstractRadixTree<K, W>],
        f: impl Fn(&mut V, &W) -> bool + Copy,
    ) {
        // this convoluted stuff is because we don't have an InPlaceMergeStateRef for Vec
        // so we convert into a smallvec, perform the ops there, then convert back.
        let mut tmp = Vec::new();
        std::mem::swap(self.children_mut(), &mut tmp);
        let mut t = SmallVec::<[Self; 0]>::from_vec(tmp);
        InPlaceMergeStateRef::merge(&mut t, &rhs, InnerCombineOp(f, PhantomData), NoConverter);
        *self.children_mut() = t.into_vec()
    }

    /// Left combine of `self` tree with `that` tree
    ///
    /// Left means that elements that are in `self` but not in `that` are kept, but elements that
    /// are in `that` but not in `self` are dropped.
    ///
    /// For elements that are in both trees, it is possible to customize how they are combined.
    /// `f` can mutate the value of `self` in place, or return false to remove the value.
    fn left_combine_with<W: TValue>(
        &mut self,
        that: &impl AbstractRadixTree<K, W>,
        f: impl Fn(&mut V, &W) -> bool + Copy,
    ) {
        let n = common_prefix(self.prefix(), that.prefix());
        if n == self.prefix().len() && n == that.prefix().len() {
            // prefixes are identical
            if let Some(w) = that.value() {
                if let Some(v) = self.value_mut() {
                    if !f(v, w) {
                        *self.value_mut() = None;
                    }
                }
            }
            self.left_combine_children_with(that.children(), f);
        } else if n == self.prefix().len() {
            // self is a prefix of that
            let that = that.materialize_shortened(n);
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

    fn left_combine_children_with<W: TValue>(
        &mut self,
        rhs: &[impl AbstractRadixTree<K, W>],
        f: impl Fn(&mut V, &W) -> bool + Copy,
    ) {
        // this convoluted stuff is because we don't have an InPlaceMergeStateRef for Vec
        // so we convert into a smallvec, perform the ops there, then convert back.
        let mut tmp = Vec::new();
        std::mem::swap(self.children_mut(), &mut tmp);
        let mut t = SmallVec::<[Self; 0]>::from_vec(tmp);
        InPlaceMergeStateRef::merge(&mut t, &rhs, LeftCombineOp(f, PhantomData), NoConverter);
        *self.children_mut() = t.into_vec()
    }
}

pub trait AbstractRadixTree<K: TKey, V: TValue>: Sized {
    fn prefix(&self) -> &[K];
    fn value(&self) -> Option<&V>;
    fn children(&self) -> &[Self];

    type Materialized: AbstractRadixTreeMut<K, V, Materialized = Self::Materialized>;

    fn is_empty(&self) -> bool {
        self.value().is_none() && self.children().is_empty()
    }

    /// true if two maps have values at the same keys
    fn intersects<W: TValue>(&self, that: &impl AbstractRadixTree<K, W>) -> bool {
        intersects(self, that)
    }

    /// true if two maps have no values at the same keys
    fn is_disjoint<W: TValue>(&self, that: &impl AbstractRadixTree<K, W>) -> bool {
        !intersects(self, that)
    }

    /// iterate over all elements
    fn iter<'a>(&'a self) -> Iter<'a, K, V, Self>
    where
        K: 'a,
    {
        Iter::new(self, IterKey::new(self.prefix()))
    }

    /// iterate over all values - this is cheaper than iterating over elements, since it does not have to build the keys from fragments
    fn values<'a>(&'a self) -> Values<'a, K, V, Self>
    where
        K: 'a,
    {
        Values::new(self)
    }

    fn contains_key(&self, key: &[K]) -> bool {
        // if we find a tree at exactly the location, and it has a value, we have a hit
        if let FindResult::Found(tree) = find(self, key) {
            tree.value().is_some()
        } else {
            false
        }
    }

    /// true if the keys of self are a subset of the keys of that.
    ///
    /// a set is considered to be a subset of itself.
    fn is_subset<W: TValue>(&self, that: &impl AbstractRadixTree<K, W>) -> bool {
        is_subset(self, that)
    }

    fn is_superset<W: TValue>(&self, that: &impl AbstractRadixTree<K, W>) -> bool {
        is_subset(that, self)
    }

    fn materialize_shortened(&self, n: usize) -> Self::Materialized {
        assert!(n < self.prefix().len());
        Self::Materialized::new(
            self.prefix()[n..].into(),
            self.value().cloned(),
            self.children()
                .iter()
                .map(|x| x.materialize_shortened(0))
                .collect(),
        )
    }

    /// Outer combine this tree with another tree, using the given combine function
    fn outer_combine(
        &self,
        that: &impl AbstractRadixTree<K, V, Materialized = Self::Materialized>,
        f: impl Fn(&V, &V) -> Option<V> + Copy,
    ) -> Self::Materialized {
        outer_combine(self, that, f)
    }

    /// Inner combine this tree with another tree, using the given combine function
    fn inner_combine<W: TValue>(
        &self,
        that: &impl AbstractRadixTree<K, W>,
        f: impl Fn(&V, &W) -> Option<V> + Copy,
    ) -> Self::Materialized {
        inner_combine(self, that, f)
    }

    /// An iterator for all pairs with a certain prefix
    fn scan_prefix<'a>(&'a self, prefix: &'a [K]) -> Iter<'a, K, V, Self> {
        match find(self, prefix) {
            FindResult::Found(tree) => {
                let prefix = IterKey::new(prefix);
                Iter::new(tree, prefix)
            }
            FindResult::Prefix { tree, rt } => {
                let mut prefix = IterKey::new(prefix);
                let remaining = &tree.prefix()[tree.prefix().len() - rt..];
                prefix.append(remaining);
                Iter::new(tree, prefix)
            }
            FindResult::NotFound { .. } => Iter::empty(),
        }
    }
}

enum FindResult<T> {
    // Found an exact match
    Found(T),
    // found a tree for which the path is a prefix, with n remaining chars in the prefix of T
    Prefix {
        // a tree of which the searched path is a prefix
        tree: T,
        // number of remaining elements in the prefix of the tree
        rt: usize,
    },
    // did not find anything, T is the closest match, with n remaining (unmatched) in the prefix of T
    NotFound {
        // the closest match
        closest: T,
        // number of remaining elements in the prefix of the tree
        rt: usize,
        // number of remaining elements in the search prefix
        rp: usize,
    },
}

/// find a prefix in a tree. Will either return
/// - Found(tree) if we found the tree exactly,
/// - Prefix if we found a tree of which prefix is a prefix
/// - NotFound if there is no tree
fn find<'a, K: TKey, V: TValue, T: AbstractRadixTree<K, V>>(
    tree: &'a T,
    prefix: &'a [K],
) -> FindResult<&'a T> {
    let n = common_prefix(tree.prefix(), prefix);
    // remaining in prefix
    let rp = prefix.len() - n;
    // remaining in tree prefix
    let rt = tree.prefix().len() - n;
    if rp == 0 && rt == 0 {
        // direct hit
        FindResult::Found(tree)
    } else if rp == 0 {
        // tree is a subtree of prefix
        FindResult::Prefix { tree, rt }
    } else if rt == 0 {
        // prefix is a subtree of tree
        let c = &prefix[n];
        if let Ok(index) = tree.children().binary_search_by(|e| e.prefix()[0].cmp(c)) {
            let child = &tree.children()[index];
            find(child, &prefix[n..])
        } else {
            FindResult::NotFound {
                closest: tree,
                rp,
                rt,
            }
        }
    } else {
        // disjoint, but we still need to store how far we matched
        FindResult::NotFound {
            closest: tree,
            rp,
            rt,
        }
    }
}

fn materialize<T, K: TKey, V: TValue>(tree: &T) -> T::Materialized
where
    K: Clone,
    V: Clone,
    T: AbstractRadixTree<K, V>,
{
    materialize_shortened(tree, 0)
}

fn materialize_shortened<T, K: TKey, V: TValue>(tree: &T, n: usize) -> T::Materialized
where
    K: Clone,
    V: Clone,
    T: AbstractRadixTree<K, V>,
{
    assert!(n < tree.prefix().len());
    T::Materialized::new(
        tree.prefix()[n..].into(),
        tree.value().cloned(),
        tree.children().iter().map(materialize).collect(),
    )
}

/// Key for iteration
#[derive(Debug, Clone)]
pub struct IterKey<K>(Arc<Vec<K>>);

impl<K: Clone> IterKey<K> {
    fn new(root: &[K]) -> Self {
        Self(Arc::new(root.to_vec()))
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

impl<T> Borrow<[T]> for IterKey<T> {
    fn borrow(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T> core::ops::Deref for IterKey<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub struct Values<'a, K, V, T> {
    stack: Vec<(&'a T, usize)>,
    _p: PhantomData<(K, V)>,
}

impl<'a, K, V, T> Values<'a, K, V, T> {
    fn new(tree: &'a T) -> Self {
        Self {
            stack: vec![(tree, 0)],
            _p: PhantomData,
        }
    }

    fn tree(&self) -> &'a T {
        self.stack.last().unwrap().0
    }

    fn inc(&mut self) -> Option<usize> {
        let pos = &mut self.stack.last_mut().unwrap().1;
        let res = if *pos == 0 { None } else { Some(*pos - 1) };
        *pos += 1;
        res
    }
}

impl<'a, K: TKey, V: TValue, T> Iterator for Values<'a, K, V, T>
where
    T: AbstractRadixTree<K, V>,
{
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.stack.is_empty() {
            if let Some(pos) = self.inc() {
                if pos < self.tree().children().len() {
                    self.stack.push((&self.tree().children()[pos], 0));
                } else {
                    self.stack.pop();
                }
            } else if let Some(value) = self.tree().value() {
                return Some(value);
            }
        }
        None
    }
}

pub struct Iter<'a, K, V, T> {
    path: IterKey<K>,
    stack: Vec<(&'a T, usize)>,
    _v: PhantomData<V>,
}

impl<'a, K: TKey, V: TValue, T: AbstractRadixTree<K, V>> Iter<'a, K, V, T> {
    fn empty() -> Self {
        Self {
            stack: Vec::new(),
            path: IterKey::new(&[]),
            _v: PhantomData,
        }
    }

    fn new(tree: &'a T, prefix: IterKey<K>) -> Self {
        Self {
            stack: vec![(tree, 0)],
            path: prefix,
            _v: PhantomData,
        }
    }

    fn tree(&self) -> &'a T {
        self.stack.last().unwrap().0
    }

    fn inc(&mut self) -> Option<usize> {
        let pos = &mut self.stack.last_mut().unwrap().1;
        let res = if *pos == 0 { None } else { Some(*pos - 1) };
        *pos += 1;
        res
    }
}

impl<'a, K: TKey, V: TValue, T: AbstractRadixTree<K, V>> SortedByKey for Iter<'a, K, V, T> {}

impl<'a, K: TKey, V: 'a + TValue, T: AbstractRadixTree<K, V>> Iterator for Iter<'a, K, V, T> {
    type Item = (IterKey<K>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while !self.stack.is_empty() {
            if let Some(pos) = self.inc() {
                if pos < self.tree().children().len() {
                    let child = &self.tree().children()[pos];
                    self.path.append(child.prefix());
                    self.stack.push((child, 0));
                } else {
                    self.path.pop(self.tree().prefix().len());
                    self.stack.pop();
                }
            } else if let Some(value) = self.tree().value().as_ref() {
                return Some((self.path.clone(), value));
            }
        }
        None
    }
}

struct RadixTreeConverter<K, V>(PhantomData<(K, V)>);

impl<T: AbstractRadixTree<K, V>, K: TKey, V: TValue> Converter<&T, T::Materialized>
    for RadixTreeConverter<K, V>
{
    fn convert(value: &T) -> T::Materialized {
        materialize(value)
    }
}

pub fn is_subset<K: TKey, V: TValue, W: TValue>(
    l: &impl AbstractRadixTree<K, V>,
    r: &impl AbstractRadixTree<K, W>,
) -> bool {
    is_subset0(l, l.prefix(), r, r.prefix())
}

fn is_subset0<K: TKey, V: TValue, W: TValue>(
    l: &impl AbstractRadixTree<K, V>,
    l_prefix: &[K],
    r: &impl AbstractRadixTree<K, W>,
    r_prefix: &[K],
) -> bool {
    let n = common_prefix(l_prefix, r_prefix);
    if n == l_prefix.len() && n == r_prefix.len() {
        // prefixes are identical
        (!l.value().is_some() || r.value().is_some())
            && !BoolOpMergeState::merge(l.children(), r.children(), NonSubsetOp(PhantomData))
    } else if n == l_prefix.len() {
        // l is a prefix of r - shorten r_prefix
        let r_prefix = &r_prefix[n..];
        // if l has a value but not r, we found one
        // if one or more of lc are not a subset of r, we are done
        l.value().is_none()
            && l.children()
                .iter()
                .all(|lc| is_subset0(lc, lc.prefix(), r, r_prefix))
    } else if n == r_prefix.len() {
        // r is a prefix of l - shorten L_prefix
        let l_prefix = &l_prefix[n..];
        // if l is a subset of none of rc, we are done
        r.children()
            .iter()
            .any(|rc| is_subset0(l, l_prefix, rc, rc.prefix()))
    } else {
        // disjoint
        false
    }
}

fn intersects<K: TKey, V: TValue, W: TValue>(
    l: &impl AbstractRadixTree<K, V>,
    r: &impl AbstractRadixTree<K, W>,
) -> bool {
    intersects0(l, l.prefix(), r, r.prefix())
}

fn intersects0<K: TKey, V: TValue, W: TValue>(
    l: &impl AbstractRadixTree<K, V>,
    l_prefix: &[K],
    r: &impl AbstractRadixTree<K, W>,
    r_prefix: &[K],
) -> bool {
    let n = common_prefix(l_prefix, r_prefix);
    if n == l_prefix.len() && n == r_prefix.len() {
        // prefixes are identical
        (l.value().is_some() && r.value().is_some())
            || BoolOpMergeState::merge(l.children(), r.children(), IntersectOp(PhantomData))
    } else if n == l_prefix.len() {
        // l is a prefix of r
        let r_prefix = &r_prefix[n..];
        l.children()
            .iter()
            .any(|lc| intersects0(lc, lc.prefix(), r, r_prefix))
    } else if n == r_prefix.len() {
        // r is a prefix of l
        let l_prefix = &l_prefix[n..];
        r.children()
            .iter()
            .any(|rc| intersects0(l, l_prefix, rc, rc.prefix()))
    } else {
        // disjoint
        false
    }
}

/// Outer combine two trees with a function f
fn outer_combine<
    K: TKey,
    V: TValue,
    R: AbstractRadixTreeMut<K, V, Materialized = R>,
    A: AbstractRadixTree<K, V, Materialized = R>,
    B: AbstractRadixTree<K, V, Materialized = R>,
>(
    a: &A,
    b: &B,
    f: impl Fn(&V, &V) -> Option<V> + Copy,
) -> R {
    let n = common_prefix(a.prefix(), b.prefix());
    let mut prefix = a.prefix()[..n].into();
    let mut children = Vec::new();
    let mut value = None;
    if n == a.prefix().len() && n == b.prefix().len() {
        // prefixes are identical
        if let Some(a) = a.value() {
            if let Some(b) = &b.value() {
                value = f(a, b);
            }
        }
        children = VecMergeState::merge(
            a.children(),
            b.children(),
            OuterCombineOp(f, PhantomData),
            RadixTreeConverter(PhantomData),
            RadixTreeConverter(PhantomData),
        );
    } else if n == a.prefix().len() {
        // a is a prefix of b
        let b = b.materialize_shortened(n);
        value = a.value().cloned();
        children = VecMergeState::merge(
            a.children(),
            &[b],
            OuterCombineOp(f, PhantomData),
            RadixTreeConverter(PhantomData),
            RadixTreeConverter(PhantomData),
        );
    } else if n == b.prefix().len() {
        // b is a prefix of a
        let a = a.materialize_shortened(n);
        value = b.value().cloned();
        children = VecMergeState::merge(
            &[a],
            b.children(),
            OuterCombineOp(f, PhantomData),
            RadixTreeConverter(PhantomData),
            RadixTreeConverter(PhantomData),
        );
    } else {
        // disjoint
        children.push(a.materialize_shortened(n));
        children.push(b.materialize_shortened(n));
        children.sort_by_key(|x| x.prefix()[0]);
    }
    R::new(prefix, value, children)
}

/// Outer combine two trees with a function f
fn inner_combine<K: TKey, V: TValue, W: TValue, R: AbstractRadixTreeMut<K, V, Materialized = R>>(
    a: &impl AbstractRadixTree<K, V, Materialized = R>,
    b: &impl AbstractRadixTree<K, W>,
    f: impl Fn(&V, &W) -> Option<V> + Copy,
) -> R {
    let n = common_prefix(a.prefix(), b.prefix());
    let prefix = a.prefix()[..n].into();
    let mut children = Vec::<R>::new();
    let mut value = None;
    if n == a.prefix().len() && n == b.prefix().len() {
        // prefixes are identical
        if let Some(a) = a.value() {
            if let Some(b) = &b.value() {
                value = f(a, b);
            }
        }
        children = VecMergeState::merge(
            a.children(),
            b.children(),
            InnerCombineOp(f, PhantomData),
            RadixTreeConverter(PhantomData),
            NoConverter,
        );
    } else if n == a.prefix().len() {
        // a is a prefix of b
        let b = b.materialize_shortened(n);
        children = VecMergeState::merge(
            a.children(),
            &[b],
            InnerCombineOp(f, PhantomData),
            RadixTreeConverter(PhantomData),
            NoConverter,
        );
    } else if n == b.prefix().len() {
        // b is a prefix of a
        let a = a.materialize_shortened(n);
        children = VecMergeState::merge(
            &[a],
            b.children(),
            InnerCombineOp(f, PhantomData),
            RadixTreeConverter(PhantomData),
            NoConverter,
        );
    } else {
        // disjoint
    }
    R::new(prefix, value, children)
}

struct IntersectOp<T>(PhantomData<T>);

impl<'a, K, V, W, I> MergeOperation<I> for IntersectOp<(K, V, W)>
where
    K: TKey,
    V: TValue,
    W: TValue,
    I: MergeStateMut,
    I::A: AbstractRadixTree<K, V>,
    I::B: AbstractRadixTree<K, W>,
{
    fn cmp(&self, a: &I::A, b: &I::B) -> Ordering {
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
        let take = intersects(a, b);
        m.advance_a(1, take)?;
        m.advance_b(1, false)
    }
}
struct NonSubsetOp<V>(PhantomData<V>);

impl<'a, K, V, W, I> MergeOperation<I> for NonSubsetOp<(K, V, W)>
where
    K: TKey,
    V: TValue,
    W: TValue,
    I: MergeStateMut,
    I::A: AbstractRadixTree<K, V>,
    I::B: AbstractRadixTree<K, W>,
{
    fn cmp(&self, a: &I::A, b: &I::B) -> Ordering {
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
struct OuterCombineOp<F, P>(F, PhantomData<P>);

impl<'a, F, K, V, A, B, C, R> MergeOperation<InPlaceMergeStateRef<'a, A, B, C>>
    for OuterCombineOp<F, (K, V)>
where
    K: TKey,
    V: TValue,
    F: Fn(&mut V, &V) -> bool + Copy,
    A: Array<Item = R>,
    B: AbstractRadixTree<K, V, Materialized = R>,
    C: Converter<&'a B, A::Item>,
    R: AbstractRadixTreeMut<K, V, Materialized = R>,
{
    fn cmp(&self, a: &A::Item, b: &B) -> Ordering {
        a.prefix()[0].cmp(&b.prefix()[0])
    }
    fn from_a(&self, m: &mut InPlaceMergeStateRef<'a, A, B, C>, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut InPlaceMergeStateRef<'a, A, B, C>, n: usize) -> EarlyOut {
        m.advance_b(n, true)
    }
    fn collision(&self, m: &mut InPlaceMergeStateRef<'a, A, B, C>) -> EarlyOut {
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

impl<'a, F, K, V, A, B, R>
    MergeOperation<VecMergeState<'a, A, B, R, RadixTreeConverter<K, V>, RadixTreeConverter<K, V>>>
    for OuterCombineOp<F, ()>
where
    K: TKey,
    V: TValue,
    A: AbstractRadixTree<K, V, Materialized = R>,
    B: AbstractRadixTree<K, V, Materialized = R>,
    F: Fn(&V, &V) -> Option<V> + Copy,
    R: AbstractRadixTreeMut<K, V, Materialized = R>,
{
    fn cmp(&self, a: &A, b: &B) -> Ordering {
        a.prefix()[0].cmp(&b.prefix()[0])
    }
    fn from_a(
        &self,
        m: &mut VecMergeState<'a, A, B, R, RadixTreeConverter<K, V>, RadixTreeConverter<K, V>>,
        n: usize,
    ) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(
        &self,
        m: &mut VecMergeState<'a, A, B, R, RadixTreeConverter<K, V>, RadixTreeConverter<K, V>>,
        n: usize,
    ) -> EarlyOut {
        m.advance_b(n, true)
    }
    fn collision(
        &self,
        m: &mut VecMergeState<'a, A, B, R, RadixTreeConverter<K, V>, RadixTreeConverter<K, V>>,
    ) -> EarlyOut {
        let a = m.a.next().unwrap();
        let b = m.b.next().unwrap();
        let res: R = outer_combine(a, b, self.0);
        if !res.is_empty() {
            m.r.push(res);
        }
        Some(())
    }
}

/// In place intersection operation
struct InnerCombineOp<F, P>(F, PhantomData<P>);

impl<'a, K, V, W, F, I, R> MergeOperation<I> for InnerCombineOp<F, (K, V, W)>
where
    K: TKey,
    V: TValue,
    W: TValue,
    F: Fn(&mut V, &W) -> bool + Copy,
    I: MutateInput<A = R>,
    I::B: AbstractRadixTree<K, W>,
    R: AbstractRadixTreeMut<K, V, Materialized = R>,
{
    fn cmp(&self, a: &I::A, b: &I::B) -> Ordering {
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

impl<'a, F, K, V, W, A, B, R>
    MergeOperation<VecMergeState<'a, A, B, R, RadixTreeConverter<K, V>, NoConverter>>
    for InnerCombineOp<F, W>
where
    K: TKey,
    V: TValue,
    W: TValue,
    A: AbstractRadixTree<K, V, Materialized = R>,
    B: AbstractRadixTree<K, W>,
    R: AbstractRadixTreeMut<K, V, Materialized = R>,
    F: Fn(&V, &W) -> Option<V> + Copy,
{
    fn cmp(&self, a: &A, b: &B) -> Ordering {
        a.prefix()[0].cmp(&b.prefix()[0])
    }
    fn from_a(
        &self,
        m: &mut VecMergeState<'a, A, B, R, RadixTreeConverter<K, V>, NoConverter>,
        n: usize,
    ) -> EarlyOut {
        m.advance_a(n, false)
    }
    fn from_b(
        &self,
        m: &mut VecMergeState<'a, A, B, R, RadixTreeConverter<K, V>, NoConverter>,
        n: usize,
    ) -> EarlyOut {
        m.advance_b(n, false)
    }
    fn collision(
        &self,
        m: &mut VecMergeState<'a, A, B, R, RadixTreeConverter<K, V>, NoConverter>,
    ) -> EarlyOut {
        let a = m.a.next().unwrap();
        let b = m.b.next().unwrap();
        let res = inner_combine(a, b, self.0);
        if !res.is_empty() {
            m.r.push(res);
        }
        Some(())
    }
}

/// In place intersection operation
struct LeftCombineOp<F, P>(F, PhantomData<P>);

impl<'a, K, V, W, F, I, R> MergeOperation<I> for LeftCombineOp<F, (K, V, W)>
where
    K: TKey,
    V: TValue,
    W: TValue,
    F: Fn(&mut V, &W) -> bool + Copy,
    I: MutateInput<A = R>,
    I::B: AbstractRadixTree<K, W>,
    R: AbstractRadixTreeMut<K, V, Materialized = R>,
{
    fn cmp(&self, a: &I::A, b: &I::B) -> Ordering {
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
    use maplit::btreeset;
    use quickcheck::*;

    impl Arbitrary for RadixTree<u8, ()> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let t: Vec<String> = Arbitrary::arbitrary(g);
            t.iter().map(|x| (x.as_bytes().to_vec(), ())).collect()
        }
    }

    impl TestSamples<Vec<u8>, bool> for RadixTree<u8, ()> {
        fn samples(&self, res: &mut BTreeSet<Vec<u8>>) {
            res.insert(vec![]);
            for (k, _) in self.iter() {
                let a = k.as_ref().to_vec();
                let mut b = a.clone();
                let mut c = a.clone();
                b.push(0);
                c.pop();
                res.insert(a);
                res.insert(b);
                res.insert(c);
            }
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

        fn is_subset_sample(a: Reference, b: Reference) -> bool {
            let a = r2t(&a);
            let b = r2t(&b);
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

    // #[test]
    // fn values_iter() {
    //     let elems = &["abc", "ab", "a", "ba"];
    //     let tree = elems
    //         .iter()
    //         .map(|x| (x.as_bytes(), (*x).to_owned()))
    //         .collect::<RadixTree<_, _>>();
    //     for x in tree.values() {
    //         println!("{}", x);
    //     }
    //     for (k, v) in tree.iter() {
    //         println!("{:?} {}", k, v);
    //     }
    // }

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
        for (key, _) in res.scan_prefix("aa".as_bytes()) {
            println!("{:?}", std::str::from_utf8(key.as_ref()).unwrap());
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

    #[test]
    fn is_subset_sample1() {
        let a = r2t(&btreeset! { vec![1]});
        let b = r2t(&btreeset! {});
        println!("a.is_subset(b): {}", a.is_subset(&b));
        assert!(binary_property_test(&a, &b, a.is_subset(&b), |a, b| !a | b));
    }
}

fn offset_from<T, U>(base: *const T, p: *const U) -> usize {
    let base = base as usize;
    let p = p as usize;
    assert!(p >= base);
    p - base
}

fn location<T>(x: &T) -> usize {
    (x as *const T) as usize
}
