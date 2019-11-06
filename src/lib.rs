#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
#[macro_use]
extern crate maplit;

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[cfg(test)]
#[macro_use]
mod test_macros;

mod flip_buffer;

mod binary_merge;
mod merge_state;

mod vec_seq;
mod total_vec_seq;

mod vec_set;
mod total_vec_set;

mod vec_map;
mod total_vec_map;

mod range_set;
mod sonic_reducer;
mod dedup;
mod iterators;

pub use vec_seq::*;
pub use vec_set::*;
pub use vec_map::*;
pub use total_vec_seq::*;
pub use total_vec_set::*;
pub use total_vec_map::*;
pub use iterators::*;
pub use range_set::*;
