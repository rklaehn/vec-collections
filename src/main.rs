#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::fmt;
use std::fmt::Display;
use std::io::{self, BufRead};
use std::str::FromStr;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod checks;

#[cfg(test)]
extern crate quickcheck;

enum Bound<'a, T> {
    Empty,
    Unbound,
    Open(&'a T),
    Closed(&'a T),
}

impl<'a, T> Bound<'a, T> {

    fn mk_bound(value: &'a T, included: bool) -> Bound<'a, T> {
        if included { Bound::Closed(value) } else { Bound::Open(value) }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Interval<T: Eq> {
    Empty,
    All,
    Point(T),
    Above(T, bool),
    Below(T, bool),
    Bounded(T, bool, T, bool),
}

// let emptyRe = Regex::new(r"^ *\\( *Ø *\\) *$").unwrap();
// let whitespace = Regex::new(r"\w+").unwrap();

impl<T: Eq> fmt::Display for Interval<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn lower_bound(included: bool) -> &'static str {
            if included {
                "["
            } else {
                "("
            }
        }
        fn upper_bound(included: bool) -> &'static str {
            if included {
                "]"
            } else {
                ")"
            }
        }

        match self {
            Interval::Empty => write!(f, "(Ø)"),
            Interval::All => write!(f, "(-∞, ∞)"),
            Interval::Point(value) => write!(f, "[{}]", value),
            Interval::Above(value, included) => {
                write!(f, "{}{}, ∞)", lower_bound(*included), value)
            }
            Interval::Below(value, included) => {
                write!(f, "(-∞, {}{}", value, upper_bound(*included))
            }
            Interval::Bounded(min, min_included, max, max_included) => write!(
                f,
                "{}{}, {}{}",
                lower_bound(*min_included),
                min,
                max,
                upper_bound(*max_included)
            ),
        }
    }
}

impl<T: Eq> Interval<T> {
    pub fn at(value: T) -> Interval<T> {
        Interval::Point(value)
    }
    pub fn above(value: T) -> Interval<T> {
        Interval::Above(value, false)
    }
    pub fn at_or_above(value: T) -> Interval<T> {
        Interval::Above(value, true)
    }
    pub fn below(value: T) -> Interval<T> {
        Interval::Below(value, false)
    }
    pub fn at_or_below(value: T) -> Interval<T> {
        Interval::Below(value, true)
    }
}

impl<T: Ord> Interval<T> {

    fn lower_bound<'a>(self: &'a Interval<T>) -> Bound<'a, T> {
        match self {
            Interval::All => Bound::Unbound,
            Interval::Empty => Bound::Empty,
            Interval::Point(x) => Bound::Closed(x),
            Interval::Above(x, x_i) => Bound::mk_bound(x, *x_i),
            Interval::Below(_, _) => Bound::Unbound,
            Interval::Bounded(min, min_i, _, _) => Bound::mk_bound(min, *min_i),
        }
    }

    fn upper_bound<'a>(self: &'a Interval<T>) -> Bound<'a, T> {
        match self {
            Interval::All => Bound::Unbound,
            Interval::Empty => Bound::Empty,
            Interval::Point(x) => Bound::Closed(x),
            Interval::Below(x, x_i) => Bound::mk_bound(x, *x_i),
            Interval::Above(_, _) => Bound::Unbound,
            Interval::Bounded(_, _, max, max_i) => Bound::mk_bound(max, *max_i)
        }
    }

    pub fn intersects(self: &Interval<T>, that: Interval<T>) -> bool {
        false
    }

    pub fn contains(self: &Interval<T>, value: T) -> bool {
        match self {
            Interval::Empty => false,
            Interval::All => true,
            Interval::Point(x) => *x == value,
            Interval::Above(x, x_i) => value > *x || (*x_i && (value == *x)),
            Interval::Below(x, x_i) => value > *x || (*x_i && (value == *x)),
            Interval::Bounded(min, min_i, max, max_i) => {
                (value > *min || (*min_i && (value == *min)))
                    && (value < *max || (*max_i && (value == *max)))
            }
        }
    }

    pub fn range(
        min: T,
        min_included: bool,
        max: T,
        max_included: bool,
    ) -> Result<Interval<T>, String> {
        if min < max {
            Ok(Interval::Bounded(min, min_included, max, max_included))
        } else if min == max && min_included && max_included {
            Ok(Interval::Point(min))
        } else {
            Err(String::from("Error"))
        }
    }
}

impl<T: Ord + FromStr + Display> FromStr for Interval<T> {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref nullRe: Regex = Regex::new(r"^ *\( *Ø *\) *$").unwrap();
            static ref singleRe: Regex = Regex::new(r"^ *\[ *([^,]+) *\] *$").unwrap();
            static ref pairRe: Regex =
                Regex::new(r"^ *(\[|\() *(.+?) *, *(.+?) *(\]|\)) *$").unwrap();
        }

        fn to_value<T: FromStr>(text: &str) -> Result<T, String> {
            text.parse::<T>()
                .map_err(|_err| String::from("Parse error!"))
        }

        if nullRe.is_match(s) {
            Ok(Interval::Empty)
        } else {
            match singleRe.captures(s) {
                Some(captures) => to_value(captures.get(1).unwrap().as_str()).map(Interval::Point),
                None => match pairRe.captures(s) {
                    Some(captures) => {
                        let left = captures.get(1).unwrap().as_str();
                        let x = captures.get(2).unwrap().as_str();
                        let y = captures.get(3).unwrap().as_str();
                        let right = captures.get(4).unwrap().as_str();
                        match (left, x, y, right) {
                            ("(", "-∞", "∞", ")") => Ok(Interval::All),
                            ("(", "-∞", max, ")") => to_value(max).map(Interval::below),
                            ("(", "-∞", max, "]") => to_value(max).map(Interval::at_or_below),
                            ("(", min, "∞", ")") => to_value(min).map(Interval::above),
                            ("[", min, "∞", ")") => to_value(min).map(Interval::at_or_above),
                            ("(", min, max, ")") => to_value(min).and_then(|min| {
                                to_value(max)
                                    .and_then(|max| Interval::range(min, false, max, false))
                            }),
                            ("(", min, max, "]") => to_value(min).and_then(|min| {
                                to_value(max).and_then(|max| Interval::range(min, false, max, true))
                            }),
                            ("[", min, max, ")") => to_value(min).and_then(|min| {
                                to_value(max).and_then(|max| Interval::range(min, true, max, false))
                            }),
                            ("[", min, max, "]") => to_value(min).and_then(|min| {
                                to_value(max).and_then(|max| Interval::range(min, true, max, true))
                            }),
                            _ => Err(String::from("Parse error!")),
                        }
                    }
                    None => Err(String::from("Parse error!")),
                },
            }
        }
    }
}

fn main() {
    let i1: Interval<i32> = Interval::Empty;
    let i2: Interval<i32> = Interval::at(0);
    let x = " ( Ø ) ".parse::<Interval<i32>>().unwrap();
    let y = "[0,1)".parse::<Interval<i32>>().unwrap();
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
