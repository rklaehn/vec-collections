use crate::merge_state::InPlaceMergeState;
use crate::{
    binary_merge::{EarlyOut, MergeOperation},
    dedup::sort_and_dedup,
    iterators::SortedIter,
    merge_state::{BoolOpMergeState, MergeStateMut, SmallVecMergeState},
};
#[cfg(feature = "serde")]
use core::marker::PhantomData;
use core::{
    cmp::Ordering,
    fmt, hash,
    hash::Hash,
    iter::FromIterator,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign},
};
#[cfg(feature = "serde")]
use serde::{
    de::{Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
use smallvec::{Array, SmallVec};
use std::collections::BTreeSet;

struct SetUnionOp;
struct SetIntersectionOp;
struct SetXorOp;
struct SetDiffOpt;

/// A set backed by a [SmallVec] of elements.
///
/// `A` the underlying storage. This must be an array. The size of this array is the maximum size this collection
/// can hold without allocating. VecSet is just a wrapper around a [SmallVec], so it does not have additional memory
/// overhead.
///
/// Sets support comparison operations (
/// [is_disjoint](#method.is_disjoint),
/// [is_subset](#method.is_subset),
/// [is_superset](#method.is_superset)
/// ) and set combine operations
/// (
/// [bitand](https://doc.rust-lang.org/std/ops/trait.BitAnd.html),
/// [bitor](https://doc.rust-lang.org/std/ops/trait.BitOr.html),
/// [bitxor](https://doc.rust-lang.org/std/ops/trait.BitXor.html),
/// [sub](https://doc.rust-lang.org/std/ops/trait.Sub.html)
/// ).
/// They also support in place operations
/// (
/// [bitand_assign](https://doc.rust-lang.org/std/ops/trait.BitAndAssign.html),
/// [bitor_assign](https://doc.rust-lang.org/std/ops/trait.BitOrAssign.html),
/// [bitxor_assign](https://doc.rust-lang.org/std/ops/trait.BitXorAssign.html),
/// [sub_assign](https://doc.rust-lang.org/std/ops/trait.SubAssign.html)
/// ) that will try to avoid allocations.
///
/// # Creation
///
/// The best way to create a VecSet is to use FromIterator, via collect.
/// ```
/// use vec_collections::VecSet;
/// let a: VecSet<[u32; 4]> = (0..4).collect(); // does not allocate
/// ```
///
/// # General usage
/// ```
/// use vec_collections::VecSet;
/// let a: VecSet<[u32; 4]> = (0..4).collect(); // does not allocate
/// let b: VecSet<[u32; 2]> = (4..6).collect(); // does not allocate
/// println!("{}", a.is_disjoint(&b)); // true
/// let c = &a | &b; // underlying smallvec will spill over to the heap
/// println!("{}", c.contains(&5)); // true
/// ```
///
/// # In place operations
/// ```
/// use vec_collections::VecSet;
/// let mut a: VecSet<[u32; 4]> = (0..4).collect(); // does not allocate
/// let b: VecSet<[u32; 4]> = (2..6).collect(); // does not allocate
/// a &= b; // in place intersection, will yield 2..4, will not allocate
/// println!("{}", a.contains(&3)); // true
/// ```
///
/// # Accessing the elements as a slice
///
/// Since a VecSet is a succinct collection, you can get a reference to the contents as a slice.
///
/// ## Example: choosing a random element
/// ```
/// use vec_collections::VecSet;
/// use rand::seq::SliceRandom;
/// let mut a: VecSet<[u32; 4]> = (0..4).collect(); // does not allocate
/// let mut rng = rand::thread_rng();
/// let e = a.as_ref().choose(&mut rng).unwrap();
/// println!("{}", e);
/// ```
///
/// [SmallVec]: https://docs.rs/smallvec/1.4.1/smallvec/struct.SmallVec.html
#[derive(Default)]
pub struct VecSet<A: Array>(SmallVec<A>);

/// Type alias for a [VecSet](struct.VecSet) with up to 2 elements with inline storage.
///
/// This is a good default, since for usize sized types, 2 is the max you can fit in without making the struct larger.
pub type VecSet2<T> = VecSet<[T; 2]>;

impl<T: fmt::Debug, A: Array<Item = T>> fmt::Debug for VecSet<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<T: Clone, A: Array<Item = T>> Clone for VecSet<A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Hash, A: Array<Item = T>> Hash for VecSet<A> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: PartialEq, A: Array<Item = T>> PartialEq for VecSet<A> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq, A: Array<Item = T>> Eq for VecSet<A> {}

impl<T: PartialOrd, A: Array<Item = T>> PartialOrd for VecSet<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Ord, A: Array<Item = T>> Ord for VecSet<A> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<A: Array> VecSet<A> {
    /// Private because it does not check the invariants.
    pub(crate) fn new_unsafe(a: SmallVec<A>) -> Self {
        Self(a)
    }
    /// A set with a single element.
    pub fn single(value: A::Item) -> Self {
        let mut res = SmallVec::new();
        res.push(value);
        Self(res)
    }
    /// The empty set.
    pub fn empty() -> Self {
        Self::new_unsafe(SmallVec::new())
    }
    /// An iterator that returns references to the items of this set in sorted order
    pub fn iter(&self) -> SortedIter<core::slice::Iter<A::Item>> {
        SortedIter::new(self.0.iter())
    }
    /// The underlying memory as a slice.
    fn as_slice(&self) -> &[A::Item] {
        &self.0
    }
    /// The number of elements in the set.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Shrink the underlying SmallVec<T> to fit.
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }
    /// true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Returns the wrapped SmallVec.
    pub fn into_inner(self) -> SmallVec<A> {
        self.0
    }
}

impl<A: Array> VecSet<A>
where
    A::Item: Ord,
{
    /// insert an element.
    ///
    /// The time complexity of this is O(N), so building a large set using single element inserts will be slow!
    /// Prefer using [from_iter](from_iter) when building a large VecSet from elements.
    pub fn insert(&mut self, that: A::Item) {
        match self.0.binary_search(&that) {
            Ok(index) => self.0[index] = that,
            Err(index) => self.0.insert(index, that),
        }
    }

    /// Remove an element.
    ///
    /// The time complexity of this is O(N), so removing many elements using single element removes inserts will be slow!
    /// Prefer using [retain](retain) when removing a large number of elements.
    pub fn remove(&mut self, that: &A::Item) {
        if let Ok(index) = self.0.binary_search(&that) {
            self.0.remove(index);
        };
    }

    /// Retain all elements matching a predicate.
    pub fn retain<F: FnMut(&A::Item) -> bool>(&mut self, mut f: F) {
        self.0.retain(|entry| f(entry))
    }

    /// true if this set has no common elements with another set.
    pub fn is_disjoint<B: Array<Item = A::Item>>(&self, that: &VecSet<B>) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetIntersectionOp)
    }

    /// true if this set is a subset of another set.
    ///
    /// A set is considered to be a subset of itself.
    pub fn is_subset<B: Array<Item = A::Item>>(&self, that: &VecSet<B>) -> bool {
        !BoolOpMergeState::merge(&self.0, &that.0, SetDiffOpt)
    }

    /// true if this set is a superset of another set.
    ///
    /// A set is considered to be a superset of itself.
    pub fn is_superset<B: Array<Item = A::Item>>(&self, that: &VecSet<B>) -> bool {
        that.is_subset(self)
    }

    /// true if this set contains the item.
    ///
    /// Time complexity is O(log N). Binary search.
    pub fn contains(&self, value: &A::Item) -> bool {
        self.0.binary_search(value).is_ok()
    }

    /// creates a set from a vec.
    ///
    /// Will sort and deduplicate the vector using a stable merge sort, so worst case time complexity
    /// is O(N log N). However, this will be faster for an already partially sorted vector.
    ///
    /// Note that the backing memory of the vector might be reused, so if this is a large vector containing
    /// lots of duplicates, it is advisable to call shrink_to_fit on the resulting set.
    fn from_vec(vec: Vec<A::Item>) -> Self {
        let mut vec = vec;
        vec.sort();
        vec.dedup();
        Self::new_unsafe(SmallVec::from_vec(vec))
    }
}

impl<'a, A: Array> IntoIterator for &'a VecSet<A> {
    type Item = &'a A::Item;
    type IntoIter = SortedIter<core::slice::Iter<'a, A::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<A: Array> IntoIterator for VecSet<A> {
    type Item = A::Item;
    type IntoIter = SortedIter<smallvec::IntoIter<A>>;

    fn into_iter(self) -> Self::IntoIter {
        SortedIter::new(self.0.into_iter())
    }
}

impl<A: Array> Into<Vec<A::Item>> for VecSet<A> {
    fn into(self) -> Vec<A::Item> {
        self.0.into_vec()
    }
}

impl<A: Array> Into<SmallVec<A>> for VecSet<A> {
    fn into(self) -> SmallVec<A> {
        self.into_inner()
    }
}

impl<T: Ord + Clone, Arr: Array<Item = T>, B: Array<Item = T>> BitAnd<&VecSet<B>> for &VecSet<Arr> {
    type Output = VecSet<Arr>;
    fn bitand(self, that: &VecSet<B>) -> Self::Output {
        Self::Output::new_unsafe(SmallVecMergeState::merge(
            &self.0,
            &that.0,
            SetIntersectionOp,
        ))
    }
}

impl<T: Ord + Clone, A: Array<Item = T>, B: Array<Item = T>> BitOr<&VecSet<B>> for &VecSet<A> {
    type Output = VecSet<A>;
    fn bitor(self, that: &VecSet<B>) -> Self::Output {
        Self::Output::new_unsafe(SmallVecMergeState::merge(&self.0, &that.0, SetUnionOp))
    }
}

impl<T: Ord + Clone, A: Array<Item = T>, B: Array<Item = T>> BitXor<&VecSet<B>> for &VecSet<A> {
    type Output = VecSet<A>;
    fn bitxor(self, that: &VecSet<B>) -> Self::Output {
        Self::Output::new_unsafe(SmallVecMergeState::merge(&self.0, &that.0, SetXorOp))
    }
}

impl<T: Ord + Clone, A: Array<Item = T>, B: Array<Item = T>> Sub<&VecSet<B>> for &VecSet<A> {
    type Output = VecSet<A>;
    fn sub(self, that: &VecSet<B>) -> Self::Output {
        Self::Output::new_unsafe(SmallVecMergeState::merge(&self.0, &that.0, SetDiffOpt))
    }
}

impl<T: Ord, A: Array<Item = T>, B: Array<Item = T>> BitAndAssign<VecSet<B>> for VecSet<A> {
    fn bitand_assign(&mut self, that: VecSet<B>) {
        InPlaceMergeState::merge(&mut self.0, that.0, SetIntersectionOp);
    }
}

impl<T: Ord, A: Array<Item = T>, B: Array<Item = T>> BitOrAssign<VecSet<B>> for VecSet<A> {
    fn bitor_assign(&mut self, that: VecSet<B>) {
        InPlaceMergeState::merge(&mut self.0, that.0, SetUnionOp);
    }
}

impl<T: Ord, A: Array<Item = T>, B: Array<Item = T>> BitXorAssign<VecSet<B>> for VecSet<A> {
    fn bitxor_assign(&mut self, that: VecSet<B>) {
        InPlaceMergeState::merge(&mut self.0, that.0, SetXorOp);
    }
}

impl<T: Ord, A: Array<Item = T>, B: Array<Item = T>> SubAssign<VecSet<B>> for VecSet<A> {
    fn sub_assign(&mut self, that: VecSet<B>) {
        InPlaceMergeState::merge(&mut self.0, that.0, SetDiffOpt);
    }
}

impl<A: Array> AsRef<[A::Item]> for VecSet<A> {
    fn as_ref(&self) -> &[A::Item] {
        self.as_slice()
    }
}

impl<T: Ord, A: Array<Item = T>> From<Vec<T>> for VecSet<A> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}

/// Provides a way to create a VecSet from a BTreeSet without having to sort again
impl<T: Ord, A: Array<Item = T>> From<BTreeSet<T>> for VecSet<A> {
    fn from(value: BTreeSet<T>) -> Self {
        Self::new_unsafe(value.into_iter().collect())
    }
}

/// Builds the set from an iterator.
///
/// Uses a heuristic to deduplicate while building the set, so the intermediate storage will never be more
/// than twice the size of the resulting set. This is the most efficient way to build a large VecSet, significantly
/// more efficient than single element insertion.
///
/// Worst case performance is O(log(n)^2 * n), but performance for already partially sorted collections will be
/// significantly better. For a fully sorted collection, performance will be O(n).
impl<T: Ord, A: Array<Item = T>> FromIterator<T> for VecSet<A> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = sort_and_dedup(iter.into_iter());
        vec.shrink_to_fit();
        Self::new_unsafe(vec)
    }
}

impl<T: Ord, A: Array<Item = T>> Extend<T> for VecSet<A> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        *self |= Self::from_iter(iter);
    }
}

impl<T: Ord, I: MergeStateMut<A = T, B = T>> MergeOperation<I> for SetUnionOp {
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, true)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.advance_a(1, true)?;
        m.advance_b(1, false)
    }
}

impl<T: Ord, I: MergeStateMut<A = T, B = T>> MergeOperation<I> for SetIntersectionOp {
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, false)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, false)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.advance_a(1, true)?;
        m.advance_b(1, false)
    }
}

impl<T: Ord, I: MergeStateMut<A = T, B = T>> MergeOperation<I> for SetDiffOpt {
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, false)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.advance_a(1, false)?;
        m.advance_b(1, false)
    }
}

impl<T: Ord, I: MergeStateMut<A = T, B = T>> MergeOperation<I> for SetXorOp {
    fn cmp(&self, a: &T, b: &T) -> Ordering {
        a.cmp(b)
    }
    fn from_a(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_a(n, true)
    }
    fn from_b(&self, m: &mut I, n: usize) -> EarlyOut {
        m.advance_b(n, true)
    }
    fn collision(&self, m: &mut I) -> EarlyOut {
        m.advance_a(1, false)?;
        m.advance_b(1, false)
    }
}

#[cfg(feature = "serde")]
impl<A: Array> Serialize for VecSet<A>
where
    A::Item: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_seq(Some(self.len()))?;
        for item in self.iter() {
            state.serialize_element(&item)?;
        }
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, A: Array> Deserialize<'de> for VecSet<A>
where
    A::Item: Deserialize<'de> + Ord,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(VecSetVisitor {
            phantom: PhantomData,
        })
    }
}

#[cfg(feature = "serde")]
struct VecSetVisitor<A> {
    phantom: PhantomData<A>,
}

#[cfg(feature = "serde")]
impl<'de, A: Array> Visitor<'de> for VecSetVisitor<A>
where
    A::Item: Deserialize<'de> + Ord,
{
    type Value = VecSet<A>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a sequence")
    }

    fn visit_seq<B>(self, mut seq: B) -> Result<Self::Value, B::Error>
    where
        B: SeqAccess<'de>,
    {
        let len = seq.size_hint().unwrap_or(0);
        let mut values = SmallVec::with_capacity(len);

        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        values.sort();
        values.dedup();
        Ok(VecSet(values))
    }
}

// impl<T: Ord> BitAnd for VecSet<T> {
//     type Output = VecSet<T>;
//     fn bitand(mut self, that: Self) -> Self::Output {
//         self &= that;
//         self
//     }
// }

// impl<T: Ord> BitOr for VecSet<T> {
//     type Output = VecSet<T>;
//     fn bitor(mut self, that: Self) -> Self::Output {
//         self |= that;
//         self
//     }
// }

// impl<T: Ord> BitXor for VecSet<T> {
//     type Output = VecSet<T>;
//     fn bitxor(mut self, that: Self) -> Self::Output {
//         self ^= that;
//         self
//     }
// }

// impl<T: Ord + Default + Copy> VecSet<T> {
//     pub fn union_with(&mut self, that: &VecSet<T>) {
//         InPlaceMergeState::merge(&mut self.0, &that.0, SetUnionOp());
//     }

//     pub fn intersection_with(&mut self, that: &VecSet<T>) {
//         InPlaceMergeState::merge(&mut self.0, &that.0, SetIntersectionOp());
//     }

//     pub fn xor_with(&mut self, that: &VecSet<T>) {
//         InPlaceMergeState::merge(&mut self.0, &that.0, SetIntersectionOp());
//     }

//     pub fn difference_with(&mut self, that: &VecSet<T>) {
//         InPlaceMergeState::merge(&mut self.0, &that.0, SetDiffOpt());
//     }
// }

#[cfg(test)]
mod test {
    use super::*;
    use crate::obey::*;
    use num_traits::PrimInt;
    use quickcheck::*;

    impl<T: Arbitrary + Ord + Copy + Default + fmt::Debug> Arbitrary for VecSet<[T; 2]> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Self::from_vec(Arbitrary::arbitrary(g))
        }
    }

    impl<E: PrimInt> TestSamples<E, bool> for VecSet<[E; 2]> {
        fn samples(&self, res: &mut BTreeSet<E>) {
            res.insert(E::min_value());
            for x in self.0.iter().cloned() {
                res.insert(x - E::one());
                res.insert(x);
                res.insert(x + E::one());
            }
            res.insert(E::max_value());
        }

        fn at(&self, elem: E) -> bool {
            self.contains(&elem)
        }
    }

    type Test = VecSet<[i64; 2]>;
    type Reference = BTreeSet<i64>;

    quickcheck! {

        #[cfg(feature = "serde")]
        fn serde_roundtrip(reference: Test) -> bool {
            let bytes = serde_json::to_vec(&reference).unwrap();
            let deser = serde_json::from_slice(&bytes).unwrap();
            reference == deser
        }

        fn is_disjoint_sample(a: Test, b: Test) -> bool {
            binary_property_test(&a, &b, a.is_disjoint(&b), |a, b| !(a & b))
        }

        fn is_subset_sample(a: Test, b: Test) -> bool {
            binary_property_test(&a, &b, a.is_subset(&b), |a, b| !a | b)
        }

        fn union_sample(a: Test, b: Test) -> bool {
            binary_element_test(&a, &b, &a | &b, |a, b| a | b)
        }

        fn intersection_sample(a: Test, b: Test) -> bool {
            binary_element_test(&a, &b, &a & &b, |a, b| a & b)
        }

        fn xor_sample(a: Test, b: Test) -> bool {
            binary_element_test(&a, &b, &a ^ &b, |a, b| a ^ b)
        }

        fn diff_sample(a: Test, b: Test) -> bool {
            binary_element_test(&a, &b, &a - &b, |a, b| a & !b)
        }

        fn union(a: Reference, b: Reference) -> bool {
            let mut a1: Test = a.iter().cloned().collect();
            let b1: Test = b.iter().cloned().collect();
            let r2 = &a1 | &b1;
            a1 |= b1;
            println!("{:?} {:?}", a, b);
            let expected: Vec<i64> = a.union(&b).cloned().collect();
            let actual: Vec<i64> = a1.into();
            let actual2: Vec<i64> = r2.into();
            expected == actual && expected == actual2
        }

        fn intersection(a: Reference, b: Reference) -> bool {
            let mut a1: Test = a.iter().cloned().collect();
            let b1: Test = b.iter().cloned().collect();
            let r2 = &a1 & &b1;
            a1 &= b1;
            let expected: Vec<i64> = a.intersection(&b).cloned().collect();
            let actual: Vec<i64> = a1.into();
            let actual2: Vec<i64> = r2.into();
            expected == actual && expected == actual2
        }

        fn xor(a: Reference, b: Reference) -> bool {
            let mut a1: Test = a.iter().cloned().collect();
            let b1: Test = b.iter().cloned().collect();
            let r2 = &a1 ^ &b1;
            a1 ^= b1;
            let expected: Vec<i64> = a.symmetric_difference(&b).cloned().collect();
            let actual: Vec<i64> = a1.into();
            let actual2: Vec<i64> = r2.into();
            expected == actual && expected == actual2
        }

        fn difference(a: Reference, b: Reference) -> bool {
            let mut a1: Test = a.iter().cloned().collect();
            let b1: Test = b.iter().cloned().collect();
            let r2 = &a1 - &b1;
            a1 -= b1;
            let expected: Vec<i64> = a.difference(&b).cloned().collect();
            let actual: Vec<i64> = a1.into();
            let actual2: Vec<i64> = r2.into();
            expected == actual && expected == actual2
        }

        fn is_disjoint(a: Reference, b: Reference) -> bool {
            let a1: Test = a.iter().cloned().collect();
            let b1: Test = b.iter().cloned().collect();
            let actual = a1.is_disjoint(&b1);
            let expected = a.is_disjoint(&b);
            expected == actual
        }

        fn is_subset(a: Reference, b: Reference) -> bool {
            let a1: Test = a.iter().cloned().collect();
            let b1: Test = b.iter().cloned().collect();
            let actual = a1.is_subset(&b1);
            let expected = a.is_subset(&b);
            expected == actual
        }

        fn contains(a: Reference, b: i64) -> bool {
            let a1: Test = a.iter().cloned().collect();
            let expected = a.contains(&b);
            let actual = a1.contains(&b);
            expected == actual
        }
    }

    bitop_assign_consistent!(Test);
    set_predicate_consistent!(Test);
    bitop_symmetry!(Test);
    bitop_empty!(Test);
}
