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

use interval::*;
use interval_seq::*;

fn main() {
    let i1: Interval<i32> = Interval::Empty;
    let i2: Interval<i32> = Interval::at(0);
    let x = " ( Ø ) ".parse::<Interval<i32>>().unwrap();
    let y = "[0,1)".parse::<Interval<i32>>().unwrap();
    let z: IntervalSeq<i64> = IntervalSeq::at_or_above(10) & IntervalSeq::at_or_below(11);
    let w: IntervalSeq<i64> = IntervalSeq::from(Interval::range(10, false, 20, false).unwrap());
    println!("{:?}", z);
    println!("{}", z);
    println!("{:?}", w);
    println!("{}", w);
    println!("Hello, world! {} {} {} {}", i1, i2, x, y);
    for line in io::stdin().lock().lines() {
        let text = line.unwrap();
        let res = text
            .parse::<Interval<i32>>()
            .map(|x| format!("{}", x))
            .unwrap_or(String::from("Error"));
        println!("{}", res);
    }
}
