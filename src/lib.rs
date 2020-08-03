//! This crate provides collections (sets and maps) that wrap [SmallVec](smallvec::SmallVec).
//!
//! # Motivation
//!
//! It happens very frequently that you have collections that have on average just a very small number of elements. If you know
//! the maximum size or even the maximum _typical_ size in advance, you can use this crate to store such collections without allocations.
//!
//! Another very frequent pattern is that you have a possibly large collection that is being created once and then used readonly for
//! a long time. E.g. some lookup tables. In these cases, ease of adding individual new elements is less important than compact in-memory
//! representation. This crate provides succinct collections that have only a very small constant overhead over the contents of the collections.
//!
//! # Performance
//!
//! Performance for bulk creation as well as lookup is better than [BTreeSet]/[BTreeMap] and comparable with [HashSet] for types with a cheap Ord instance, like
//! e.g. primitive types. Performance for insertion or removal of individual elements to/from large collections is very bad, however.
//!
//! # Collections overview
//!
//! ## [VecSet]
//!
//! Provides a set backed by a [SmallVec] of elements.
//!
//! ## [TotalVecSet]
//!
//! A [VecSet] with an additional flag so it can support negation. This way it is possible to represent e.g. the set of all u64 except 1.
//!
//! ## [VecMap]
//!
//! Provides a map backed by a [SmallVec] of key value pairs.
//!
//! ## [TotalVecMap]
//!
//! A [VecMap] with an additional default value, so lookup is a total function.
//!
//! [VecSet]: vec_et::VecSet
//! [VecMap]: vec_map::VecMap
//! [TotalVecSet]: total_vec_set::TotalVecSet
//! [TotalVecMap]: total_vec_map::TotalVecMap
//! [BTreeSet]: std::collections::BTreeSet
//! [BTreeMap]: std::collections::BTreeMap
//! [HashSet]: std::collections::HashSet
//! [HashMap]: std::collections::HashMap
//! [SmallVec]: smallvec::SmallVec
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

mod binary_merge;
mod merge_state;

mod total_vec_set;
mod vec_set;

mod total_vec_map;
mod vec_map;

mod dedup;
mod iterators;

#[cfg(test)]
mod obey;

mod small_vec_builder;

pub use total_vec_map::*;
pub use total_vec_set::*;
pub use vec_map::*;
pub use vec_set::*;
