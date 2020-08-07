use crate::{
    binary_merge::{EarlyOut, MergeOperation},
    merge_state::SmallVecMergeState,
    vec_map::VecMap,
};
use num_traits::{Bounded, One, Zero};
use smallvec::Array;
use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::Debug,
    hash::Hash,
    ops::{Add, Div, Index, Mul, Neg, Sub},
};

/// A [VecMap] with default value.
///
/// Having a default value means that the mapping is a total function from K to V, hence the name.
///
/// [VecMap]: struct.VecMap.html
pub struct TotalVecMap<V, A: Array>(VecMap<A>, V);

/// Type alias for a [TotalVecMap](struct.TotalVecMap) with up to 1 mappings with inline storage.
pub type TotalVecMap1<K, V> = TotalVecMap<V, [(K, V); 1]>;

impl<K: Clone, V: Clone, A: Array<Item = (K, V)>> Clone for TotalVecMap<V, A> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<K: Hash, V: Hash, A: Array<Item = (K, V)>> Hash for TotalVecMap<V, A> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<K: PartialEq, V: PartialEq, A: Array<Item = (K, V)>> PartialEq for TotalVecMap<V, A> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K: Eq, V: Eq, A: Array<Item = (K, V)>> Eq for TotalVecMap<V, A> {}

impl<K: PartialOrd, V: PartialOrd, A: Array<Item = (K, V)>> PartialOrd for TotalVecMap<V, A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (&self.0, &self.1).partial_cmp(&(&other.0, &other.1))
    }
}

impl<K: Ord, V: Ord, A: Array<Item = (K, V)>> Ord for TotalVecMap<V, A> {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.0, &self.1).cmp(&(&other.0, &other.1))
    }
}

impl<K, V: Default, A: Array<Item = (K, V)>> Default for TotalVecMap<V, A> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

impl<K, V: Eq, A: Array<Item = (K, V)>> TotalVecMap<V, A> {
    /// Creates a total vec map, given a vec map and a default value.
    ///
    /// Mappings in the map that map to the default value will be removed in order to have
    /// a unique representation.
    pub fn new(map: VecMap<A>, default: V) -> Self {
        let mut entries = map;
        // ensure canonical representation!
        entries.retain(|(_, v)| *v != default);
        Self(entries, default)
    }
}

impl<K, V, A: Array<Item = (K, V)>> TotalVecMap<V, A> {
    /// Creates a constant mapping from any K to the given V.
    pub fn constant(value: V) -> Self {
        Self(VecMap::default(), value)
    }

    /// Returns all non-default mappings as a `VecMap<A>`.
    pub fn non_default_mappings(&self) -> &VecMap<A> {
        &self.0
    }
}

impl<K: Debug, V: Debug, A: Array<Item = (K, V)>> Debug for TotalVecMap<V, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TotalVecMap")
            .field("values", &self.0)
            .field("default", &self.1)
            .finish()
    }
}

/// Creates a constant mapping from any K to the given V.
impl<K, V, A: Array<Item = (K, V)>> From<V> for TotalVecMap<V, A> {
    fn from(value: V) -> Self {
        Self::constant(value)
    }
}

impl<K, V: Bounded, A: Array<Item = (K, V)>> Bounded for TotalVecMap<V, A> {
    fn min_value() -> Self {
        V::min_value().into()
    }
    fn max_value() -> Self {
        V::max_value().into()
    }
}

impl<K: Ord + Clone, V: Add<Output = V> + Eq + Clone, A: Array<Item = (K, V)>> Add
    for TotalVecMap<V, A>
{
    type Output = TotalVecMap<V, A>;

    fn add(self, that: Self) -> Self::Output {
        self.combine_ref(&that, |a, b| a.clone() + b.clone())
    }
}

impl<K: Ord + Clone, V: Sub<Output = V> + Eq + Clone, A: Array<Item = (K, V)>> Sub
    for TotalVecMap<V, A>
{
    type Output = Self;

    fn sub(self, that: Self) -> Self::Output {
        self.combine_ref(&that, |a, b| a.clone() - b.clone())
    }
}

impl<K: Ord + Clone, V: Neg<Output = V> + Eq + Clone, A: Array<Item = (K, V)>> Neg
    for TotalVecMap<V, A>
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.map_values(|a| -a.clone())
    }
}

impl<K: Ord + Clone, V: Mul<Output = V> + Eq + Clone, A: Array<Item = (K, V)>> Mul
    for TotalVecMap<V, A>
{
    type Output = TotalVecMap<V, A>;

    fn mul(self, that: Self) -> Self::Output {
        self.combine_ref(&that, |a, b| a.clone() * b.clone())
    }
}

impl<K: Ord + Clone, V: Div<Output = V> + Eq + Clone, A: Array<Item = (K, V)>> Div
    for TotalVecMap<V, A>
{
    type Output = TotalVecMap<V, A>;

    fn div(self, that: Self) -> Self::Output {
        self.combine_ref(&that, |a, b| a.clone() / b.clone())
    }
}

impl<K: Ord + Clone, V: Zero + Eq + Clone, A: Array<Item = (K, V)>> Zero for TotalVecMap<V, A> {
    fn zero() -> Self {
        V::zero().into()
    }
    fn is_zero(&self) -> bool {
        self.0.is_empty() && self.1.is_zero()
    }
}

impl<K: Ord + Clone, V: One + Eq + Clone, A: Array<Item = (K, V)>> One for TotalVecMap<V, A> {
    fn one() -> Self {
        V::one().into()
    }
    fn is_one(&self) -> bool {
        self.0.is_empty() && self.1.is_one()
    }
}

struct CombineOp<F, V> {
    f: F,
    a_default: V,
    b_default: V,
    r_default: V,
}

/// a fast combine op is an op where we know that the default for both a and b is the neutral element of the operation
struct FastCombineOp<'a, F, V> {
    f: F,
    r_default: &'a V,
}

type PairMergeState2<'a, Arr> =
    SmallVecMergeState<'a, <Arr as Array>::Item, <Arr as Array>::Item, Arr>;

impl<'a, K: Ord + Clone, V: Eq, F: Fn(&V, &V) -> V, Arr: Array<Item = (K, V)>>
    MergeOperation<PairMergeState2<'a, Arr>> for CombineOp<F, &'a V>
{
    fn cmp(&self, a: &(K, V), b: &(K, V)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut PairMergeState2<'a, Arr>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, a)) = m.a.next() {
                let result = (self.f)(a, self.b_default);
                if result != *self.r_default {
                    m.r.push((k.clone(), result))
                }
            }
        }
        Some(())
    }
    fn from_b(&self, m: &mut PairMergeState2<'a, Arr>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, b)) = m.b.next() {
                let result = (self.f)(self.a_default, b);
                if result != *self.r_default {
                    m.r.push((k.clone(), result))
                }
            }
        }
        Some(())
    }
    fn collision(&self, m: &mut PairMergeState2<'a, Arr>) -> EarlyOut {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                let result = (self.f)(a, b);
                if result != *self.r_default {
                    m.r.push((k.clone(), result))
                }
            }
        }
        Some(())
    }
}

impl<'a, K: Ord + Clone, V: Eq + Clone, F: Fn(&V, &V) -> V, Arr: Array<Item = (K, V)>>
    MergeOperation<PairMergeState2<'a, Arr>> for FastCombineOp<'a, F, V>
{
    fn cmp(&self, a: &(K, V), b: &(K, V)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut PairMergeState2<'a, Arr>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, a)) = m.a.next() {
                m.r.push((k.clone(), a.clone()));
            }
        }
        Some(())
    }
    fn from_b(&self, m: &mut PairMergeState2<'a, Arr>, n: usize) -> EarlyOut {
        for _ in 0..n {
            if let Some((k, b)) = m.b.next() {
                m.r.push((k.clone(), b.clone()));
            }
        }
        Some(())
    }
    fn collision(&self, m: &mut PairMergeState2<'a, Arr>) -> EarlyOut {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                let result = (self.f)(a, b);
                if result != *self.r_default {
                    m.r.push((k.clone(), result))
                }
            }
        }
        Some(())
    }
}

impl<K: Ord + Clone, V: Eq, A: Array<Item = (K, V)>> TotalVecMap<V, A> {
    /// combine a total map with another total map, using a function that takes value references
    pub fn combine_ref<F: Fn(&V, &V) -> V>(&self, that: &Self, f: F) -> Self {
        let r_default = f(&self.1, &that.1);
        let op = CombineOp {
            f,
            a_default: &self.1,
            b_default: &that.1,
            r_default: &r_default,
        };
        let r = SmallVecMergeState::merge(self.0.as_ref(), that.0.as_ref(), op);
        Self(VecMap::new(r), r_default)
    }
}

impl<K: Ord + Clone, V: Ord + Clone, A: Array<Item = (K, V)>> TotalVecMap<V, A> {
    pub fn supremum(&self, that: &Self) -> Self {
        self.combine_ref(that, |a, b| std::cmp::max(a, b).clone())
    }
    pub fn infimum(&self, that: &Self) -> Self {
        self.combine_ref(that, |a, b| std::cmp::min(a, b).clone())
    }
}

/// not sure if I can even use fast_combine in rust
#[allow(dead_code)]
impl<K: Ord + Clone, V: Eq + Clone, A: Array<Item = (K, V)>> TotalVecMap<V, A> {
    pub(crate) fn fast_combine<F: Fn(&V, &V) -> V>(
        &self,
        that: &TotalVecMap<V, A>,
        f: F,
    ) -> TotalVecMap<V, A> {
        let r_default = f(&self.1, &that.1);
        let op = FastCombineOp {
            f,
            r_default: &r_default,
        };
        let r = SmallVecMergeState::merge(self.0.as_ref(), that.0.as_ref(), op);
        Self(VecMap::new(r), r_default)
    }
}

impl<K: Clone, V: Eq, A: Array<Item = (K, V)>> TotalVecMap<V, A> {
    pub fn map_values<W: Eq, F: Fn(&V) -> W, B: Array<Item = (K, W)>>(
        &self,
        f: F,
    ) -> TotalVecMap<W, B> {
        let default = f(&self.1);
        let elements: smallvec::SmallVec<B> = self
            .0
            .slice_iter()
            .filter_map(|entry| {
                let w = f(&entry.1);
                if w != default {
                    Some((entry.0.clone(), w))
                } else {
                    None
                }
            })
            .collect();
        TotalVecMap(VecMap::new(elements), default)
    }
}

impl<K: Ord + 'static, Q: ?Sized, V, A: Array<Item = (K, V)>> Index<&Q> for TotalVecMap<V, A>
where
    K: Borrow<Q>,
    Q: Ord,
{
    type Output = V;

    /// Lookup. Time complexity is O(log N), where N is the number of non-default elements
    fn index(&self, key: &Q) -> &V {
        self.0.get(key).unwrap_or(&self.1)
    }
}

// we don't implement IndexMut since that would allow changing a value to the default and all sorts of other nasty things!
#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::*;
    use std::collections::{BTreeMap, BTreeSet};

    type Ref = (BTreeMap<i32, i32>, i32);
    type Test = TotalVecMap1<i32, i32>;

    fn from_ref(r: Ref) -> Test {
        let (elements, default) = r;
        Test::new(elements.clone().into(), default)
    }

    impl<K: Arbitrary + Ord, V: Arbitrary + Eq> Arbitrary for TotalVecMap1<K, V> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            TotalVecMap::new(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g))
        }
    }

    fn combine_reference<F: Fn(i32, i32) -> i32>(a: &Ref, b: &Ref, f: F) -> Ref {
        let (a, ad) = a.clone();
        let (b, bd) = b.clone();
        let rd = f(ad, bd);
        let mut r: BTreeMap<i32, i32> = BTreeMap::default();
        let mut keys: BTreeSet<i32> = BTreeSet::new();
        keys.extend(a.keys());
        keys.extend(b.keys());
        for key in keys {
            let value = f(*a.get(&key).unwrap_or(&ad), *b.get(&key).unwrap_or(&bd));
            r.insert(key, value);
        }
        (r, rd)
    }

    quickcheck! {

        fn index(a: Ref, key: i32) -> bool {
            let x = from_ref(a.clone());
            let (elements, default) = a;
            let expected = elements.get(&key).cloned().unwrap_or(default);
            let actual = x[&key];
            expected == actual
        }

        fn supremum(a: Ref, b: Ref) -> bool {
            let expected = from_ref(combine_reference(&a, &b, |a, b| std::cmp::max(a, b)));
            let a1 = from_ref(a.clone());
            let b1 = from_ref(b.clone());
            let actual = a1.supremum(&b1);
            expected == actual
        }

        fn infimum(a: Ref, b: Ref) -> bool {
            let expected = from_ref(combine_reference(&a, &b, |a, b| std::cmp::min(a, b)));
            let a1 = from_ref(a.clone());
            let b1 = from_ref(b.clone());
            let actual = a1.infimum(&b1);
            expected == actual
        }
    }
}
