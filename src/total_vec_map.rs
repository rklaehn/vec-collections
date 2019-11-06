use crate::binary_merge::MergeOperation;
use crate::merge_state::VecMergeState;
use crate::vec_map::VecMap;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::ops::Index;

#[derive(Hash, Debug, Clone, Eq, PartialEq, Default)]
pub struct TotalArrayMap<K, V>(VecMap<K, V>, V);

impl<K, V: Eq> TotalArrayMap<K, V> {
    pub fn new(map: VecMap<K, V>, default: V) -> Self {
        let mut entries = map;
        // ensure canonical representation!
        entries.retain(|(_, v)| *v != default);
        Self(entries, default)
    }
}

impl<K, V> TotalArrayMap<K, V> {
    pub fn constant(value: V) -> Self {
        Self(VecMap::default(), value)
    }

    pub fn as_slice(&self) -> &[(K, V)] {
        self.0.as_slice()
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

type PairMergeState<'a, K, V> = VecMergeState<'a, (K, V), (K, V), (K, V)>;

impl<'a, K: Ord + Clone, V: Eq, F: Fn(&V, &V) -> V>
    MergeOperation<(K, V), (K, V), PairMergeState<'a, K, V>> for CombineOp<F, &'a V>
{
    fn cmp(&self, a: &(K, V), b: &(K, V)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut PairMergeState<'a, K, V>, n: usize) {
        for _ in 0..n {
            if let Some((k, a)) = m.a.next() {
                let result = (self.f)(a, self.b_default);
                if result != *self.r_default {
                    m.r.push((k.clone(), result))
                }
            }
        }
    }
    fn from_b(&self, m: &mut PairMergeState<'a, K, V>, n: usize) {
        for _ in 0..n {
            if let Some((k, b)) = m.b.next() {
                let result = (self.f)(self.a_default, b);
                if result != *self.r_default {
                    m.r.push((k.clone(), result))
                }
            }
        }
    }
    fn collision(&self, m: &mut PairMergeState<'a, K, V>) {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                let result = (self.f)(a, b);
                if result != *self.r_default {
                    m.r.push((k.clone(), result))
                }
            }
        }
    }
}

impl<'a, K: Ord + Clone, V: Eq + Clone, F: Fn(&V, &V) -> V>
    MergeOperation<(K, V), (K, V), PairMergeState<'a, K, V>> for FastCombineOp<'a, F, V>
{
    fn cmp(&self, a: &(K, V), b: &(K, V)) -> Ordering {
        a.0.cmp(&b.0)
    }
    fn from_a(&self, m: &mut PairMergeState<'a, K, V>, n: usize) {
        for _ in 0..n {
            if let Some((k, a)) = m.a.next() {
                m.r.push((k.clone(), a.clone()));
            }
        }
    }
    fn from_b(&self, m: &mut PairMergeState<'a, K, V>, n: usize) {
        for _ in 0..n {
            if let Some((k, b)) = m.b.next() {
                m.r.push((k.clone(), b.clone()));
            }
        }
    }
    fn collision(&self, m: &mut PairMergeState<'a, K, V>) {
        if let Some((k, a)) = m.a.next() {
            if let Some((_, b)) = m.b.next() {
                let result = (self.f)(a, b);
                if result != *self.r_default {
                    m.r.push((k.clone(), result))
                }
            }
        }
    }
}

impl<K: Ord + Clone, V: Eq> TotalArrayMap<K, V> {
    pub fn combine<F: Fn(&V, &V) -> V>(&self, that: &Self, f: F) -> Self {
        let r_default = f(&self.1, &that.1);
        let op = CombineOp {
            f,
            a_default: &self.1,
            b_default: &that.1,
            r_default: &r_default,
        };
        let r = VecMergeState::merge(self.as_slice(), that.as_slice(), op);
        Self(VecMap::from_sorted_vec(r), r_default)
    }
}

impl<K: Ord + Clone, V: Ord + Clone> TotalArrayMap<K, V> {
    pub fn supremum(&self, that: &Self) -> Self {
        self.combine(that, |a, b| if a > b { a.clone() } else { b.clone() })
    }
    pub fn infimum(&self, that: &Self) -> Self {
        self.combine(that, |a, b| if a < b { a.clone() } else { b.clone() })
    }
}

/// not sure if I can even use fast_combine in rust
#[allow(dead_code)]
impl<K: Ord + Clone, V: Eq + Clone> TotalArrayMap<K, V> {
    pub(crate) fn fast_combine<F: Fn(&V, &V) -> V>(
        &self,
        that: &TotalArrayMap<K, V>,
        f: F,
    ) -> TotalArrayMap<K, V> {
        let r_default = f(&self.1, &that.1);
        let op = FastCombineOp {
            f,
            r_default: &r_default,
        };
        let r = VecMergeState::merge(self.as_slice(), that.as_slice(), op);
        Self(VecMap::from_sorted_vec(r), r_default)
    }
}

impl<K: Clone, V: Eq> TotalArrayMap<K, V> {
    pub fn map_values<W: Eq, F: Fn(&V) -> W>(&self, f: F) -> TotalArrayMap<K, W> {
        let default = f(&self.1);
        let elements: Vec<(K, W)> = self
            .0
            .iter()
            .filter_map(|entry| {
                let w = f(&entry.1);
                if w != default {
                    Some((entry.0.clone(), w))
                } else {
                    None
                }
            })
            .collect();
        TotalArrayMap(VecMap::from_sorted_vec(elements), default)
    }
}

impl<K: Ord, Q: ?Sized, V> Index<&Q> for TotalArrayMap<K, V>
where
    K: Borrow<Q>,
    Q: Ord,
{
    type Output = V;

    fn index(&self, key: &Q) -> &V {
        self.0.get(key).unwrap_or(&self.1)
    }
}

mod alga_instances {
    use super::*;
    use alga::general::*;

    impl<K: Ord + Clone, V: AbstractMagma<Additive> + Eq> AbstractMagma<Additive>
        for TotalArrayMap<K, V>
    {
        fn operate(&self, that: &Self) -> Self {
            self.combine(that, V::operate)
        }
    }

    impl<K, V: Identity<Additive>> Identity<Additive> for TotalArrayMap<K, V> {
        fn identity() -> Self {
            TotalArrayMap::constant(V::identity())
        }
    }

    impl<K: Clone, V: TwoSidedInverse<Additive> + Eq> TwoSidedInverse<Additive>
        for TotalArrayMap<K, V>
    {
        fn two_sided_inverse(&self) -> Self {
            self.map_values(V::two_sided_inverse)
        }
    }

    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractSemigroup<Additive> + Eq> AbstractSemigroup<Additive> for TotalArrayMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractMonoid<Additive> + Eq> AbstractMonoid<Additive> for TotalArrayMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractQuasigroup<Additive> + Eq> AbstractQuasigroup<Additive> for TotalArrayMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractLoop<Additive> + Eq> AbstractLoop<Additive> for TotalArrayMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractGroup<Additive> + Eq> AbstractGroup<Additive> for TotalArrayMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractGroupAbelian<Additive> + Eq> AbstractGroupAbelian<Additive> for TotalArrayMap<K, V> {}

    impl<K: Ord + Clone, V: AbstractMagma<Multiplicative> + Eq> AbstractMagma<Multiplicative>
        for TotalArrayMap<K, V>
    {
        fn operate(&self, that: &Self) -> Self {
            self.combine(that, V::operate)
        }
    }

    impl<K, V: Identity<Multiplicative>> Identity<Multiplicative> for TotalArrayMap<K, V> {
        fn identity() -> Self {
            TotalArrayMap::constant(V::identity())
        }
    }

    impl<K: Clone, V: TwoSidedInverse<Multiplicative> + Eq> TwoSidedInverse<Multiplicative>
        for TotalArrayMap<K, V>
    {
        fn two_sided_inverse(&self) -> Self {
            self.map_values(V::two_sided_inverse)
        }
    }

    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractSemigroup<Multiplicative> + Eq> AbstractSemigroup<Multiplicative> for TotalArrayMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractMonoid<Multiplicative> + Eq> AbstractMonoid<Multiplicative> for TotalArrayMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractQuasigroup<Multiplicative> + Eq> AbstractQuasigroup<Multiplicative> for TotalArrayMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractLoop<Multiplicative> + Eq> AbstractLoop<Multiplicative> for TotalArrayMap<K, V> {}
    #[rustfmt::skip] impl<K: Ord + Clone, V: AbstractGroup<Multiplicative> + Eq> AbstractGroup<Multiplicative> for TotalArrayMap<K, V> {}
}

// we don't implement IndexMut since that would allow changing a value to the default and all sorts of other nasty things!
#[cfg(test)]
mod tests {
    use super::*;
    use alga::general::*;
    use quickcheck::*;
    use std::collections::{BTreeMap, BTreeSet};

    type Ref = (BTreeMap<i32, i32>, i32);
    type Test = TotalArrayMap<i32, i32>;

    fn from_ref(r: Ref) -> Test {
        let (elements, default) = r;
        Test::new(elements.clone().into(), default)
    }

    impl<K: Arbitrary + Ord, V: Arbitrary + Eq> Arbitrary for TotalArrayMap<K, V> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            TotalArrayMap::new(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g))
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
