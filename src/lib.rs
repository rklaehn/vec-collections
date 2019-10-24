#[cfg(test)]
extern crate quickcheck;

mod array_seq;
mod binary_merge;
mod total_array_seq;

pub use array_seq::*;
pub use total_array_seq::*;
// pub use array_set::*;

pub use binary_merge::*;
