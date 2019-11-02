use crate::{ArraySeq, ArraySet};
use std::borrow::Borrow;
use std::cmp::Ordering;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Entry<K, V>(K, V);

impl<K, V> Borrow<K> for Entry<K,V> {
    fn borrow(&self) -> &K {
        &self.0
    }
}

impl<K: Ord, V: Eq> PartialOrd for Entry<K, V> {
    fn partial_cmp(&self, that: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&that.0))
    }
}

impl<K: Ord, V: Eq> Ord for Entry<K, V> {
    fn cmp(&self, that: &Self) -> Ordering {
        self.0.cmp(&that.0)
    }
}

#[derive(Hash, Clone)]
struct ArrayMap<K, V>(ArraySet<Entry<K, V>>);

struct MergeOp();

impl<K: Ord, V> ArrayMap<K, V> {
    pub fn merge(that: ArrayMap<K, V>) {}

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let elements = self.0.as_slice();
        match elements.binary_search_by(|p| p.0.borrow().borrow().cmp(key)) {
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
        match elements.binary_search_by(|p| p.0.borrow().borrow().cmp(key)) {
            Ok(index) => Some(&mut elements[index].1),
            Err(_) => None,
        }
    }
}
