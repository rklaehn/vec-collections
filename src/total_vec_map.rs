//! A map with default value, backed by a `SmallVec<(K, V)>` of key value pairs.
//!
//! Having a default value means that the mapping is a total function from K to V, hence the name.
use crate::binary_merge::{EarlyOut, MergeOperation};
use crate::merge_state::SmallVecMergeState;
use crate::vec_map::VecMap;
use num_traits::{Bounded, One, Zero};
use smallvec::Array;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::{Add, Div, Index, Mul, Neg, Sub};

#[derive(Hash, Clone, Eq, PartialEq, Default)]
pub struct TotalVecMap<K, V, A: Array<Item = (K, V)> = [(K, V); 2]>(VecMap<K, V, A>, V);

impl<K, V: Eq, A: Array<Item = (K, V)>> TotalVecMap<K, V, A> {
    /// Creates a total vec map, given a vec map and a default value.
    ///
    /// Mappings in the map that map to the default value will be removed in order to have
    /// a unique representation.
    pub fn new(map: VecMap<K, V, A>, default: V) -> Self {
        let mut entries = map;
        // ensure canonical representation!
        entries.retain(|(_, v)| *v != default);
        Self(entries, default)
    }
}

impl<K, V, A: Array<Item = (K, V)>> TotalVecMap<K, V, A> {
    /// Creates a constant mapping from any K to the given V.
    pub fn constant(value: V) -> Self {
        Self(VecMap::default(), value)
    }

    // pub fn non_default_mappings(&self) -> &VecMap<K, V> {
    //     self.0
    // }
}

impl<K: Debug, V: Debug, A: Array<Item = (K, V)>> Debug for TotalVecMap<K, V, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TotalVecMap")
            .field("values", &self.0)
            .field("default", &self.1)
            .finish()
    }
}

/// Creates a constant mapping from any K to the given V.
impl<K, V, A: Array<Item = (K, V)>> From<V> for TotalVecMap<K, V, A> {
    fn from(value: V) -> Self {
        Self::constant(value)
    }
}

impl<K, V: Bounded, A: Array<Item = (K, V)>> Bounded for TotalVecMap<K, V, A> {
    fn min_value() -> Self {
        V::min_value().into()
    }
    fn max_value() -> Self {
        V::max_value().into()
    }
}

impl<K: Ord + Clone, V: Add<Output = V> + Eq + Clone> Add for TotalVecMap<K, V> {
    type Output = TotalVecMap<K, V>;

    fn add(self, that: Self) -> Self::Output {
        self.combine_ref(&that, |a, b| a.clone() + b.clone())
    }
}

impl<K: Ord + Clone, V: Sub<Output = V> + Eq + Clone> Sub for TotalVecMap<K, V> {
    type Output = Self;

    fn sub(self, that: Self) -> Self::Output {
        self.combine_ref(&that, |a, b| a.clone() - b.clone())
    }
}

impl<K: Ord + Clone, V: Neg<Output = V> + Eq + Clone> Neg for TotalVecMap<K, V> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.map_values(|a| -a.clone())
    }
}

impl<K: Ord + Clone, V: Mul<Output = V> + Eq + Clone> Mul for TotalVecMap<K, V> {
    type Output = TotalVecMap<K, V>;

    fn mul(self, that: Self) -> Self::Output {
        self.combine_ref(&that, |a, b| a.clone() * b.clone())
    }
}

impl<K: Ord + Clone, V: Div<Output = V> + Eq + Clone> Div for TotalVecMap<K, V> {
    type Output = TotalVecMap<K, V>;

    fn div(self, that: Self) -> Self::Output {
        self.combine_ref(&that, |a, b| a.clone() / b.clone())
    }
}

impl<K: Ord + Clone, V: Zero + Eq + Clone> Zero for TotalVecMap<K, V> {
    fn zero() -> Self {
        V::zero().into()
    }
    fn is_zero(&self) -> bool {
        self.0.is_empty() && self.1.is_zero()
    }
}

impl<K: Ord + Clone, V: One + Eq + Clone> One for TotalVecMap<K, V> {
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

type PairMergeState2<'a, Arr: Array> = SmallVecMergeState<'a, Arr::Item, Arr::Item, Arr>;

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

impl<K: Ord + Clone, V: Eq> TotalVecMap<K, V> {
    /// combine a total map with another total map, using a function that takes value references
    pub fn combine_ref<F: Fn(&V, &V) -> V>(&self, that: &Self, f: F) -> Self {
        let r_default = f(&self.1, &that.1);
        let op = CombineOp {
            f,
            a_default: &self.1,
            b_default: &that.1,
            r_default: &r_default,
        };
        let r = SmallVecMergeState::merge(self.0.as_slice(), that.0.as_slice(), op);
        Self(VecMap::new(r), r_default)
    }
}

impl<K: Ord + Clone, V: Ord + Clone> TotalVecMap<K, V> {
    pub fn supremum(&self, that: &Self) -> Self {
        self.combine_ref(that, |a, b| std::cmp::max(a, b).clone())
    }
    pub fn infimum(&self, that: &Self) -> Self {
        self.combine_ref(that, |a, b| std::cmp::min(a, b).clone())
    }
}

/// not sure if I can even use fast_combine in rust
#[allow(dead_code)]
impl<K: Ord + Clone, V: Eq + Clone> TotalVecMap<K, V> {
    pub(crate) fn fast_combine<F: Fn(&V, &V) -> V>(
        &self,
        that: &TotalVecMap<K, V>,
        f: F,
    ) -> TotalVecMap<K, V> {
        let r_default = f(&self.1, &that.1);
        let op = FastCombineOp {
            f,
            r_default: &r_default,
        };
        let r = SmallVecMergeState::merge(self.0.as_slice(), that.0.as_slice(), op);
        Self(VecMap::new(r), r_default)
    }
}

impl<K: Clone, V: Eq> TotalVecMap<K, V> {
    pub fn map_values<W: Eq, F: Fn(&V) -> W>(&self, f: F) -> TotalVecMap<K, W> {
        let default = f(&self.1);
        let elements: Vec<(K, W)> = self
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
        TotalVecMap(VecMap::from_sorted_vec(elements), default)
    }
}

impl<K: Ord, Q: ?Sized, V> Index<&Q> for TotalVecMap<K, V>
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

mod alga_instances {
    use super::*;
    use alga::general::*;

    impl<K: Ord + Clone, V: AbstractMagma<Additive> + Eq> AbstractMagma<Additive>
        for TotalVecMap<K, V>
    {
        fn operate(&self, that: &Self) -> Self {
            self.combine_ref(that, V::operate)
        }
    }

    impl<K, V: Identity<Additive>> Identity<Additive> for TotalVecMap<K, V> {
        fn identity() -> Self {
            TotalVecMap::constant(V::identity())
        }
    }

    impl<K: Clone, V: TwoSidedInverse<Additive> + Eq> TwoSidedInverse<Additive> for TotalVecMap<K, V> {
        fn two_sided_inverse(&self) -> Self {
            self.map_values(V::two_sided_inverse)
        }
    }

    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractSemigroup<Additive> + Eq> AbstractSemigroup<Additive> for TotalVecMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractMonoid<Additive> + Eq> AbstractMonoid<Additive> for TotalVecMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractQuasigroup<Additive> + Eq> AbstractQuasigroup<Additive> for TotalVecMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractLoop<Additive> + Eq> AbstractLoop<Additive> for TotalVecMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractGroup<Additive> + Eq> AbstractGroup<Additive> for TotalVecMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractGroupAbelian<Additive> + Eq> AbstractGroupAbelian<Additive> for TotalVecMap<K, V> {}

    impl<K: Ord + Clone, V: AbstractMagma<Multiplicative> + Eq> AbstractMagma<Multiplicative>
        for TotalVecMap<K, V>
    {
        fn operate(&self, that: &Self) -> Self {
            self.combine_ref(that, V::operate)
        }
    }

    impl<K, V: Identity<Multiplicative>> Identity<Multiplicative> for TotalVecMap<K, V> {
        fn identity() -> Self {
            TotalVecMap::constant(V::identity())
        }
    }

    impl<K: Clone, V: TwoSidedInverse<Multiplicative> + Eq> TwoSidedInverse<Multiplicative>
        for TotalVecMap<K, V>
    {
        fn two_sided_inverse(&self) -> Self {
            self.map_values(V::two_sided_inverse)
        }
    }

    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractSemigroup<Multiplicative> + Eq> AbstractSemigroup<Multiplicative> for TotalVecMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractMonoid<Multiplicative> + Eq> AbstractMonoid<Multiplicative> for TotalVecMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractQuasigroup<Multiplicative> + Eq> AbstractQuasigroup<Multiplicative> for TotalVecMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractLoop<Multiplicative> + Eq> AbstractLoop<Multiplicative> for TotalVecMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractGroup<Multiplicative> + Eq> AbstractGroup<Multiplicative> for TotalVecMap<K, V> {}
}

// we don't implement IndexMut since that would allow changing a value to the default and all sorts of other nasty things!
#[cfg(test)]
mod tests {
    use super::*;
    use alga::general::*;
    use quickcheck::*;
    use std::collections::{BTreeMap, BTreeSet};

    type Ref = (BTreeMap<i32, i32>, i32);
    type Test = TotalVecMap<i32, i32>;

    fn from_ref(r: Ref) -> Test {
        let (elements, default) = r;
        Test::new(elements.clone().into(), default)
    }

    impl<K: Arbitrary + Ord, V: Arbitrary + Eq> Arbitrary for TotalVecMap<K, V> {
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

        fn prop_is_associative_additive(args: (Test, Test, Test)) -> bool {
            AbstractSemigroup::<Additive>::prop_is_associative(args)
        }

        fn prop_operating_identity_element_is_noop_additive(args: (Test, )) -> bool {
            AbstractMonoid::<Additive>::prop_operating_identity_element_is_noop(args)
        }

        fn prop_is_commutative(args: (Test, Test)) -> bool {
            AbstractGroupAbelian::<Additive>::prop_is_commutative(args)
        }
    }
}
