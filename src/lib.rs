#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[cfg(test)]
extern crate maplit;

extern crate sorted_iter;
pub use sorted_iter::{SortedIteratorExt, SortedPairIteratorExt};

#[cfg(test)]
#[macro_use]
mod test_macros;

#[allow(dead_code)]
mod flip_buffer;

mod binary_merge;
mod merge_state;

mod total_vec_seq;
mod vec_seq;

mod total_vec_set;
mod vec_set;

mod total_vec_map;
mod vec_map;

mod dedup;
mod iterators;
mod range_set;

#[allow(dead_code)]
mod sonic_reducer;

pub use range_set::*;
pub use total_vec_map::*;
pub use total_vec_seq::*;
pub use total_vec_set::*;
pub use vec_map::*;
pub use vec_seq::*;
pub use vec_set::*;
