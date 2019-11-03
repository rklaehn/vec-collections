#[cfg(test)]
#[macro_use]
extern crate quickcheck;

extern crate flip_buffer;

mod binary_merge;
mod merge_state;

mod array_seq;
mod total_array_seq;

mod array_set;
mod total_array_set;

mod array_map;

pub use array_seq::*;
pub use array_set::*;
pub use total_array_seq::*;
pub use total_array_set::*;

pub use array_map::*;

use binary_merge::*;
use merge_state::*;
