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

mod array_seq;
mod total_array_seq;

mod array_set;
mod total_array_set;

mod array_map;
mod total_array_map;

mod range_set;

mod sonic_reducer;

mod dedup;

mod iterators;

pub use array_seq::*;
pub use array_set::*;
pub use iterators::*;
pub use total_array_seq::*;
pub use total_array_set::*;
