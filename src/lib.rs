#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::io::{self, BufRead};
use std::str::FromStr;

mod interval;
mod interval_seq;

#[cfg(test)]
mod benches;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod checks;

#[cfg(test)]
extern crate quickcheck;

pub use interval::*;
pub use interval_seq::*;
