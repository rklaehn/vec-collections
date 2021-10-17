use std::cmp::Ordering;

use smallvec::SmallVec;

use crate::{
    binary_merge::{EarlyOut, MergeOperation},
    merge_state::{
        BoolOpMergeState, InPlaceMergeStateRef, MergeStateMut, MutateInput, SmallVecMergeState,
    },
    AbstractVecSet,
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

impl<T: Copy> Fragment<T> {
    fn key(&self) -> T {
        self.0[0]
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

impl<K: std::fmt::Debug + Ord + Copy, V> RadixTree<K, V> {
    pub fn contains_key(&self, key: &[K]) -> bool {
        self.intersects(&RadixTree::single(key, ()))
    }

    pub fn intersects<W>(&self, that: &RadixTree<K, W>) -> bool {
        Self::intersects0(self, &self.prefix, that, &that.prefix)
    }

    fn intersects0<W>(l: &Self, l_prefix: &[K], r: &RadixTree<K, W>, r_prefix: &[K]) -> bool {
        let n = common_prefix(&l_prefix, &r_prefix);
        if n == l_prefix.len() && n == r_prefix.len() {
            // prefixes are identical
            (l.value.is_some() && r.value.is_some())
                || Self::intersect_children(&l.children, &r.children)
        } else if n == l_prefix.len() {
            // l is a prefix of r
            let r_prefix = &r_prefix[n..];
            l.children
                .iter()
                .any(|lc| Self::intersects0(lc, &lc.prefix, r, r_prefix))
        } else if n == r_prefix.len() {
            // r is a prefix of l
            let l_prefix = &l_prefix[n..];
            r.children
                .iter()
                .any(|rc| Self::intersects0(l, l_prefix, rc, &rc.prefix))
        } else {
            // disjoint
            false
        }
    }

    fn intersect_children<W>(l: &[Self], r: &[RadixTree<K, W>]) -> bool {
        BoolOpMergeState::merge(l, r, IntersectOp)
    }
}

impl<K: std::fmt::Debug + Ord + Copy, V: std::fmt::Debug + Clone> RadixTree<K, V> {
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

    pub fn prefix(self, prefix: &[K]) -> Self {
        if prefix.is_empty() {
            self
        } else {
            let mut prefix1 = SmallVec::new();
            prefix1.extend_from_slice(prefix);
            prefix1.extend_from_slice(&self.prefix);
            Self {
                prefix: prefix1.into(),
                value: self.value,
                children: self.children,
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_none() && self.children.is_empty()
    }

    /// create an artificial split at offset n
    fn split(&mut self, n: usize) {
        assert!(n > 0 && n < self.prefix.len());
        let first = self.prefix[..n].into();
        let rest = self.prefix[n..].into();
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

    fn clone_shortened(&self, n: usize) -> Self {
        assert!(n < self.prefix.len());
        let mut res = self.clone();
        res.prefix = res.prefix[n..].into();
        res
    }

    pub fn union_with(&mut self, that: &RadixTree<K, V>) {
        self.combine_with(that, |l, r| {
            if l.is_none() {
                if r.is_some() {
                    *l = r.clone();
                }
            }
        })
    }

    fn combine_with(&mut self, that: &Self, f: impl Fn(&mut Option<V>, &Option<V>) + Copy) {
        let n = common_prefix(&self.prefix, &that.prefix);
        if n == self.prefix.len() && n == that.prefix.len() {
            // prefixes are identical
            f(&mut self.value, &that.value);
            self.combine_children_with(&that.children, f);
        } else if n == self.prefix.len() {
            // self is a prefix of that
            let that = that.clone_shortened(n);
            self.combine_children_with(&[that], f);
        } else if n == that.prefix.len() {
            // that is a prefix of self
            // split at the offset, then merge in that
            // we must not swap sides!
            self.split(n);
            // self now has the same prefix as that, so just repeat the code
            // from where prefixes are identical
            f(&mut self.value, &that.value);
            self.combine_children_with(&that.children, f);
        } else {
            assert!(n > 0);
            // disjoint
            self.split(n);
            self.children.push(that.clone_shortened(n));
            self.children.sort_by_key(|x| x.prefix.key());
        }
    }

    fn combine_children_with(
        &mut self,
        rhs: &[Self],
        f: impl Fn(&mut Option<V>, &Option<V>) + Copy,
    ) {
        let mut tmp = Vec::new();
        std::mem::swap(&mut self.children, &mut tmp);
        let mut t = SmallVec::<[Self; 1]>::from_vec(tmp);
        InPlaceMergeStateRef::merge(&mut t, &rhs, MergeOp(f));
        self.children = t.into_vec()
    }
}

struct IntersectOp;

impl<'a, K, V, W, I> MergeOperation<I> for IntersectOp
where
    K: Ord + Copy + std::fmt::Debug,
    I: MergeStateMut<A = RadixTree<K, V>, B = RadixTree<K, W>>,
{
    fn cmp(&self, a: &RadixTree<K, V>, b: &RadixTree<K, W>) -> Ordering {
        a.prefix.key().cmp(&b.prefix.key())
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

/// In place merge operation
struct MergeOp<F>(F);

impl<'a, F, K, V, I> MergeOperation<I> for MergeOp<F>
where
    F: Fn(&mut Option<V>, &Option<V>) + Copy,
    V: Clone + std::fmt::Debug,
    K: Ord + Copy + std::fmt::Debug,
    I: MutateInput<A = RadixTree<K, V>, B = RadixTree<K, V>>,
{
    fn cmp(&self, a: &RadixTree<K, V>, b: &RadixTree<K, V>) -> Ordering {
        a.prefix.key().cmp(&b.prefix.key())
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
        av.combine_with(bv, self.0);
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
        }
        for key in keys {
            assert!(res.contains_key(key.as_bytes()));
        }
        for key in nope {
            assert!(!res.contains_key(key.as_bytes()));
        }
    }
}
