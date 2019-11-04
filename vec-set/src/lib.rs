#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
#[macro_use]
extern crate maplit;

extern crate flip_buffer;

mod binary_merge;
mod merge_state;

mod array_seq;
mod total_array_seq;

mod array_set;
mod total_array_set;

mod array_map;
mod total_array_map;

mod sonic_reducer;

mod dedup;

pub use array_seq::*;
pub use array_set::*;
pub use total_array_seq::*;
pub use total_array_set::*;
