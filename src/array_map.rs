use crate::merge_state::VecMergeState;
use crate::{EarlyOut, MergeOperation, MergeState};
use std::borrow::Borrow;
use std::cmp::Ordering;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Entry<K, V>(K, V);

#[derive(Hash, Clone)]
struct ArrayMap<K, V>(Vec<Entry<K, V>>);

struct MapLeftUnionOp();

impl<'a, K: Ord, V, I: MergeState<Entry<K, V>, Entry<K, V>>>
    MergeOperation<'a, Entry<K, V>, Entry<K, V>, I> for MapLeftUnionOp
{
    fn cmp(&self, a: &Entry<K, V>, b: &Entry<K, V>) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_a(n)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.move_b(n)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.move_a(1)?;
        m.skip_b(1)
    }
}

// struct MapOuterJoinOp<A, B, R, F: FnMut(&A, &B) -> R>(F);

// impl<'a, K: Ord, A, B, R, F: FnMut(&A, &B) -> R, I: MergeState<Entry<K, A>, Entry<K, B>>> MergeOperation<'a, Entry<K, A>, Entry<K, B>, I> for MapOuterJoinOp<A, B, R, F> {
//     fn cmp(&self, a: &Entry<K, A>, b: &Entry<K, B>) -> Ordering {
//         a.0.cmp(&b.0)
//     }
//     fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
//         m.move_a(n)
//     }
//     fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
//         m.move_b(n)
//     }
//     fn collision(&self, m: &mut I) -> EarlyOut {
//         let r = self.0(&m.a_slice()[0].1, &m.b_slice()[0].1);
//         m.skip_a(1)?;
//         m.skip_b(1)
//     }
// }

impl<K, V> ArrayMap<K, V> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn retain<F: FnMut((&K, &V)) -> bool>(&mut self, mut f: F) {
        self.0.retain(|entry| f((&entry.0, &entry.1)))
    }

    pub fn map_values<R, F: FnMut(V) -> R>(self, mut f: F) -> ArrayMap<K, R> {
        ArrayMap::from_sorted_vec(
            self.0
                .into_iter()
                .map(|entry| Entry(entry.0, f(entry.1)))
                .collect(),
        )
    }

    fn from_sorted_vec(v: Vec<Entry<K, V>>) -> Self {
        Self(v)
    }
}

impl<K: Ord + Clone, V: Clone + Eq> ArrayMap<K, V> {
    pub fn merge(&self, that: &ArrayMap<K, V>) -> Self {
        Self::from_sorted_vec(VecMergeState::merge(
            self.0.as_slice(),
            that.0.as_slice(),
            MapLeftUnionOp(),
        ))
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let elements = self.0.as_slice();
        match elements.binary_search_by(|p| p.0.borrow().cmp(key)) {
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
        match elements.binary_search_by(|p| p.0.borrow().cmp(key)) {
            Ok(index) => Some(&mut elements[index].1),
            Err(_) => None,
        }
    }
}
