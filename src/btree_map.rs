//! Some support for operations that combine BTreeMaps and archived BTreeMaps
//!
use std::{
    borrow::Borrow,
    collections::{btree_map, BTreeMap},
};

use rkyv::collections::ArchivedBTreeMap;

use crate::OuterJoinArg;

pub enum AbstractBTreeMapIter<'a, K, V> {
    BTreeMap(btree_map::Iter<'a, K, V>),
    ArchivedBTreeMap(rkyv::collections::btree_map::Iter<'a, K, V>),
}

impl<'a, K, V> Iterator for AbstractBTreeMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            AbstractBTreeMapIter::BTreeMap(x) => x.next(),
            AbstractBTreeMapIter::ArchivedBTreeMap(x) => x.next(),
        }
    }
}

impl<K, V> AbstractBTreeMap<K, V> for BTreeMap<K, V> {
    fn iter(&self) -> AbstractBTreeMapIter<K, V> {
        AbstractBTreeMapIter::BTreeMap(self.iter())
    }

    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Ord + Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key)
    }
}

impl<K, V> AbstractBTreeMap<K, V> for ArchivedBTreeMap<K, V> {
    fn iter(&self) -> AbstractBTreeMapIter<K, V> {
        AbstractBTreeMapIter::ArchivedBTreeMap(self.iter())
    }

    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Ord + Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key)
    }
}

pub trait AbstractBTreeMap<K, V> {
    fn iter(&self) -> AbstractBTreeMapIter<K, V>;
    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Ord + Borrow<Q>,
        Q: Ord + ?Sized;

    fn outer_join<W, R, F>(&self, that: &impl AbstractBTreeMap<K, W>, f: F) -> BTreeMap<K, R>
    where
        K: Ord + Clone,
        F: Fn(OuterJoinArg<&K, &V, &W>) -> Option<R>,
    {
        let mut res = BTreeMap::new();
        for (k, v) in self.iter() {
            let arg = match that.get(k) {
                Some(w) => OuterJoinArg::Both(k, v, w),
                None => OuterJoinArg::Left(k, v),
            };
            if let Some(r) = f(arg) {
                res.insert(k.clone(), r);
            }
        }
        for (k, w) in that.iter() {
            if self.get(k).is_none() {
                let arg = OuterJoinArg::Right(k, w);
                if let Some(r) = f(arg) {
                    res.insert(k.clone(), r);
                }
            }
        }
        res
    }

    fn left_join<W, R, F>(&self, that: &impl AbstractBTreeMap<K, W>, f: F) -> BTreeMap<K, R>
    where
        K: Ord + Clone,
        F: Fn(&K, &V, Option<&W>) -> Option<R>,
    {
        let mut res = BTreeMap::new();
        for (k, v) in self.iter() {
            let w = that.get(k);
            if let Some(r) = f(k, v, w) {
                res.insert(k.clone(), r);
            }
        }
        res
    }

    fn right_join<W, R, F>(&self, that: &impl AbstractBTreeMap<K, W>, f: F) -> BTreeMap<K, R>
    where
        K: Ord + Clone,
        F: Fn(&K, Option<&V>, &W) -> Option<R>,
    {
        let mut res = BTreeMap::new();
        for (k, w) in that.iter() {
            let v = self.get(k);
            if let Some(r) = f(k, v, w) {
                res.insert(k.clone(), r);
            }
        }
        res
    }

    fn inner_join<W, R, F, A>(&self, that: &impl AbstractBTreeMap<K, W>, f: F) -> BTreeMap<K, R>
    where
        K: Ord + Clone,
        F: Fn(&K, &V, &W) -> Option<R>,
    {
        let mut res = BTreeMap::new();
        let self_iter = self.iter();
        let that_iter = that.iter();
        if self_iter.size_hint().0 < that_iter.size_hint().0 {
            for (k, v) in self.iter() {
                if let Some(w) = that.get(k) {
                    if let Some(r) = f(k, v, w) {
                        res.insert(k.clone(), r);
                    }
                }
            }
        } else {
            for (k, w) in that.iter() {
                if let Some(v) = self.get(k) {
                    if let Some(r) = f(k, v, w) {
                        res.insert(k.clone(), r);
                    }
                }
            }
        }
        res
    }
}

pub trait InPlaceRelationalOps<K, V> {
    fn inner_join_with<W, F>(&mut self, that: &impl AbstractBTreeMap<K, W>, f: F)
    where
        K: Ord,
        F: Fn(&K, &mut V, &W) -> bool;

    fn left_join_with<W, F>(&mut self, that: &impl AbstractBTreeMap<K, W>, f: F)
    where
        K: Ord,
        F: Fn(&K, &mut V, Option<&W>) -> bool;

    fn right_join_with<W, F>(&mut self, that: &impl AbstractBTreeMap<K, W>, f: F)
    where
        K: Ord + Clone,
        F: Fn(&K, Option<&mut V>, &W) -> bool;

    fn outer_join_with<W, L, R>(&mut self, that: &impl AbstractBTreeMap<K, W>, l: L, r: R)
    where
        K: Ord + Clone,
        L: Fn(&K, &mut V, Option<&W>) -> bool,
        R: Fn(&K, &W) -> Option<V>;
}

impl<K, V> InPlaceRelationalOps<K, V> for BTreeMap<K, V> {
    fn inner_join_with<W, F>(&mut self, that: &impl AbstractBTreeMap<K, W>, f: F)
    where
        K: Ord,
        F: Fn(&K, &mut V, &W) -> bool,
    {
        self.retain(|k, v| that.get(k).map(|w| f(k, v, w)).unwrap_or_default())
    }

    fn left_join_with<W, F>(&mut self, that: &impl AbstractBTreeMap<K, W>, f: F)
    where
        K: Ord,
        F: Fn(&K, &mut V, Option<&W>) -> bool,
    {
        self.retain(|k, v| f(k, v, that.get(k)))
    }

    fn right_join_with<W, F>(&mut self, that: &impl AbstractBTreeMap<K, W>, f: F)
    where
        K: Ord + Clone,
        F: Fn(&K, Option<&mut V>, &W) -> bool,
    {
        for (k, w) in that.iter() {
            if !f(k, self.get_mut(k), w) {
                self.remove(k);
            }
        }
    }

    fn outer_join_with<W, L, R>(&mut self, that: &impl AbstractBTreeMap<K, W>, l: L, r: R)
    where
        K: Ord + Clone,
        L: Fn(&K, &mut V, Option<&W>) -> bool,
        R: Fn(&K, &W) -> Option<V>,
    {
        // k in that
        for (k, w) in that.iter() {
            match self.get_mut(k) {
                Some(v) => {
                    if !l(k, v, Some(w)) {
                        self.remove(k);
                    }
                }
                None => {
                    if let Some(v) = r(k, w) {
                        self.insert(k.clone(), v);
                    }
                }
            }
        }
        // k not in that
        self.retain(|k, v| that.get(k).is_some() || l(k, v, None));
    }
}
