//! This crate provides collections (sets and maps) that wrap [SmallVec].
//!
//! # Use cases
//!
//! ## Small collections
//!
//! It happens very frequently that you have collections that have on average just a very small number of elements. If you know
//! the maximum size or even the maximum _typical_ size in advance, you can use this crate to store such collections without allocations.
//! For a larger number of elements, the underlying [SmallVec] will allocate the elements on the heap as a single allocation.
//!
//! ## Read-heavy collections
//!
//! Another very frequent pattern is that you have a possibly large collection that is being created once and then used readonly for
//! a long time. E.g. lookup tables. In these cases, ease of adding individual new elements is less important than compact in-memory
//! representation and lookup performance. This crate provides succinct collections that have only a very small constant overhead over
//! the contents of the collections.
//!
//! # Performance
//!
//! Performance for bulk creation as well as lookup is better than [BTreeMap]/[BTreeSet] and comparable with [HashMap]/[HashSet] for
//! types with a cheap [Ord] instance, like primitive types, and small to medium sizes. Performance for insertion or removal of
//! individual elements to/from large collections is bad, however. This is not the intended use case.
//!
//! # Collections overview
//!
//! ## [VecSet]
//!
//! Provides a set backed by a [SmallVec] of elements.
//!
//! ## [VecMap]
//!
//! Provides a map backed by a [SmallVec] of key value pairs.
//!
//! ## [RadixTree]
//!
//! A [RadixTree] that comes in different flavours.
//!
//! ## [TotalVecSet]
//!
//! A [VecSet] with an additional flag so it can support negation. This way it is possible to represent e.g. the set of all u64 except 1.
//!
//! ## [TotalVecMap]
//!
//! A [VecMap] with an additional default value, so lookup is a total function.
//!
//! # Unsafe
//!
//! The in place operations use unsafe code. If that is a problem for you, let me know and I can hide them behind a feature.
//!
//! [SmallVec]: https://docs.rs/smallvec/1.4.1/smallvec/struct.SmallVec.html
//! [VecSet]: struct.VecSet.html
//! [VecMap]: struct.VecMap.html
//! [TotalVecSet]: struct.TotalVecSet
//! [TotalVecMap]: struct.TotalVecMap
//! [RadixTree]: radix_tree/struct.RadixTree.html
//! [Ord]: https://doc.rust-lang.org/std/cmp/trait.Ord.html
//! [BTreeSet]: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html
//! [BTreeMap]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
//! [HashSet]: https://doc.rust-lang.org/std/collections/struct.HashSet.html
//! [HashMap]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
#[cfg(test)]
extern crate quickcheck;

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[cfg(test)]
extern crate maplit;

extern crate sorted_iter;
pub use sorted_iter::{SortedIterator, SortedPairIterator};

#[cfg(test)]
#[macro_use]
mod test_macros;

mod merge_state;

mod vec_map;
mod vec_set;

#[cfg(feature = "radixtree")]
pub mod radix_tree;

#[cfg(feature = "total")]
pub mod total_vec_map;

#[cfg(feature = "total")]
pub mod total_vec_set;

#[cfg(feature = "std_support")]
pub mod btree_map;

mod dedup;
mod iterators;

mod macros;

#[cfg(test)]
mod obey;

pub use dedup::{sort_and_dedup, sort_and_dedup_by_key};
pub use macros::*;
pub use smallvec::Array;
pub use vec_map::*;
pub use vec_set::*;
