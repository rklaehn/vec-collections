use std::iter::FromIterator;
use vec_collections::{VecMap2, VecSet2};

#[derive(Debug)]
struct Multimap<K, V>(VecMap2<K, VecSet2<V>>);

impl<K: Eq + Ord + 'static, V: Eq + Ord + Clone> Multimap<K, V> {
    fn single(key: K, value: V) -> Self {
        Multimap(vec![(key, VecSet2::single(value))].into_iter().collect())
    }
    fn combine_with(&mut self, rhs: Multimap<K, V>) {
        self.0.combine_with(rhs.0, |a, b| &a | &b)
    }
}

impl<K, V> Default for Multimap<K, V> {
    fn default() -> Self {
        Self(VecMap2::default())
    }
}

impl<K: Eq + Ord + 'static, V: Clone + Eq + Ord> FromIterator<(K, V)> for Multimap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut res = Multimap::default();
        for (k, v) in iter.into_iter() {
            res.combine_with(Self::single(k, v))
        }
        res
    }
}

#[derive(Debug)]
struct BiMultimap<K, V>(VecMap2<K, VecSet2<V>>, VecMap2<V, VecSet2<K>>);

impl<K, V> Default for BiMultimap<K, V> {
    fn default() -> Self {
        Self(VecMap2::default(), VecMap2::default())
    }
}

impl<K: Eq + Ord + Clone + 'static, V: Eq + Ord + Clone + 'static> BiMultimap<K, V> {
    fn single(key: K, value: V) -> Self {
        Self(
            vec![(key.clone(), VecSet2::single(value.clone()))]
                .into_iter()
                .collect(),
            vec![(value, VecSet2::single(key))].into_iter().collect(),
        )
    }
    fn combine_with(&mut self, rhs: BiMultimap<K, V>) {
        self.0.combine_with(rhs.0, |a, b| &a | &b);
        self.1.combine_with(rhs.1, |a, b| &a | &b);
    }
}

impl<K: Default + Clone + Eq + Ord + 'static, V: Default + Clone + Eq + Ord + 'static>
    FromIterator<(K, V)> for BiMultimap<K, V>
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut res = BiMultimap::default();
        for (k, v) in iter.into_iter() {
            res.combine_with(Self::single(k, v))
        }
        res
    }
}

fn main() {
    let x: Multimap<u32, u32> = [(0, 0), (0, 1), (1, 0)].iter().cloned().collect();
    let y: BiMultimap<u32, u32> = [(0, 0), (0, 1), (1, 0)].iter().cloned().collect();
}
