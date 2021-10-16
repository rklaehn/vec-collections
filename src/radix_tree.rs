use std::{cmp::Ordering, sync::Arc};

use smallvec::SmallVec;

use crate::{binary_merge::{EarlyOut, MergeOperation}, merge_state::{InPlaceMergeState, MutateInput}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Fragment<T>(Arc<[T]>);

impl<T> AsRef<[T]>  for Fragment<T> {
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
        Self(Arc::new([]))
    }
}

fn common_prefix<'a, T: Eq>(a: &'a [T], b: &'a [T]) -> &'a [T] {
    let max = a.len().min(b.len());
    for i in 0..max {
        if a[i] != b[i] {
            return &a[..i];
        }
    }
    &a[..max]
}

impl<T: Copy> Fragment<T> {

    fn key(&self) -> T {
        self.0[0]
    }

    fn new(value: &[T]) -> Option<Self> {
        if !value.is_empty() {
            Some(Self(value.into()))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
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

impl<K: Ord + Copy, V: Clone> RadixTree<K, V> {

    pub fn leaf(value: V) -> Self {
        Self {
            prefix: Fragment(Arc::new([])),
            value: Some(value),
            children: Default::default(),
        }
    }

    pub fn single(key: &[K], value: V) -> Self {
        Self::leaf(value).prefix(key)
    }

    pub fn prefix(self, prefix: &[K]) -> Self {
        if prefix.is_empty() {
            self
        } else {
            let mut prefix1 = Vec::new();
            prefix1.extend_from_slice(prefix);
            prefix1.extend_from_slice(self.prefix.as_ref());
            Self {
                prefix: prefix.into(),
                value: self.value,
                children: self.children,
            }
        }
    }

    fn non_empty(self) -> Option<Self> {
        if !self.is_empty() {
            Some(self)
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_none() && self.children.is_empty()
    }

    fn take(&mut self) -> Self {
        let mut res = Self::default();
        std::mem::swap(self, &mut res);
        res
    }

    pub fn combine_with(&mut self, that: &RadixTree<K, V>, f: &impl Fn(&mut Option<V>, &Option<V>)) {
        let common = common_prefix(&self.prefix, &that.prefix);
        if common.len() == self.prefix.len() && common.len() == that.prefix.len() {
            // prefixes are identical
            f(&mut self.value, &that.value);
            self.combine_children_with(&that.children, f);
        } else if common.len() == self.prefix.len() {
            // self is a prefix of that
            let rest = self.prefix[common.len()..].into();           
            let mut that = that.clone();
            that.prefix = rest;
            self.combine_children_with(&[that], f);
        } else if common.len() == that.prefix.len() {
            // that is a prefix of self 
            let rest = self.prefix[common.len()..].into();           
            let mut that = that.clone();
            std::mem::swap(self, &mut that);
            that.prefix = rest;
            self.combine_children_with(&[that], f);
        } else {
            // disjoint
            let n = common.len();
            let common = common.into();
            let lrest = self.prefix[n..].into();
            let rrest = that.prefix[n..].into();
            let mut l = self.take();
            let mut r = that.clone();
            l.prefix = lrest;
            r.prefix = rrest;            
            self.children = vec![l, r];
            self.prefix = common;
            self.children.sort_by_key(|x| x.prefix.key());
        }
    }

    fn combine_children_with(&mut self, children: &[Self], f: &impl Fn(&mut Option<V>, &Option<V>)) {
        let mut t = Vec::new();
        std::mem::swap(&mut self.children, &mut t);
        let mut t: SmallVec<[Self; 1]> = t.into();
        InPlaceMergeState::from()
    }
}

struct MergeOp<F>(F);

impl<'a, F, K, V, I> MergeOperation<I>
    for MergeOp<F>
    where
        F: Fn(&mut Option<V>, &Option<V>),
        V: Clone,
        K: Ord + Copy,
        I: MutateInput<A = RadixTree<K, V>, B = RadixTree<K, V>>
{
    fn cmp(&self, a: &RadixTree<K, V>, b: &RadixTree<K, V>) -> Ordering {
        a.prefix[0].cmp(&b.prefix[0])
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
        av.combine_with(bv, &self.0);
        let take = !av.is_empty();
        m.advance_a(1, take)?;
        m.advance_b(1, false)
    }
}