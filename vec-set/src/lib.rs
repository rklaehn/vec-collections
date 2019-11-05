#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
#[macro_use]
extern crate maplit;

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

extern crate flip_buffer;

#[cfg(test)]
#[macro_use]
mod test_macros;

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

pub use array_seq::*;
pub use array_set::*;
pub use total_array_seq::*;
pub use total_array_set::*;
