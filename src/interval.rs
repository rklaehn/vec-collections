use super::*;
use regex::Regex;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Bound<T> {
    Empty,
    Unbound,
    Open(T),
    Closed(T),
}

impl<T> Bound<T> {
    fn mk_bound(value: T, included: bool) -> Bound<T> {
        if included {
            Bound::Closed(value)
        } else {
            Bound::Open(value)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Interval<T> {
    Empty,
    All,
    Point(T),
    Above(T, bool),
    Below(T, bool),
    Bounded(T, bool, T, bool),
}

impl<T: Eq + Display> Display for Interval<T> {
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
    pub(crate) fn from_bounds(lower: Bound<T>, upper: Bound<T>) -> Interval<T> {
        match (lower, upper) {
            (Bound::Empty, Bound::Empty) => Interval::Empty,
            (Bound::Closed(x), Bound::Closed(y)) => Interval::range(x, true, y, true).unwrap(),
            (Bound::Closed(x), Bound::Open(y)) => Interval::range(x, true, y, false).unwrap(),
            (Bound::Open(x), Bound::Closed(y)) => Interval::range(x, false, y, true).unwrap(),
            (Bound::Open(x), Bound::Open(y)) => Interval::range(x, false, y, false).unwrap(),
            (Bound::Unbound, Bound::Open(y)) => Interval::below(y),
            (Bound::Unbound, Bound::Closed(y)) => Interval::at_or_below(y),
            (Bound::Open(x), Bound::Unbound) => Interval::above(x),
            (Bound::Closed(x), Bound::Unbound) => Interval::at_or_above(x),
            (Bound::Unbound, Bound::Unbound) => Interval::All,
            _ => panic!("invalid empty bound"),
        }
    }

    pub(crate) fn lower_bound(self: Interval<T>) -> Bound<T> {
        match self {
            Interval::All => Bound::Unbound,
            Interval::Empty => Bound::Empty,
            Interval::Point(x) => Bound::Closed(x),
            Interval::Above(x, x_i) => Bound::mk_bound(x, x_i),
            Interval::Below(_, _) => Bound::Unbound,
            Interval::Bounded(min, min_i, _, _) => Bound::mk_bound(min, min_i),
        }
    }

    pub(crate) fn upper_bound(self: Interval<T>) -> Bound<T> {
        match self {
            Interval::All => Bound::Unbound,
            Interval::Empty => Bound::Empty,
            Interval::Point(x) => Bound::Closed(x),
            Interval::Below(x, x_i) => Bound::mk_bound(x, x_i),
            Interval::Above(_, _) => Bound::Unbound,
            Interval::Bounded(_, _, max, max_i) => Bound::mk_bound(max, max_i),
        }
    }

    #[cfg(test)]
    fn is_valid(self: Interval<T>) -> bool {
        match self {
            Interval::Bounded(min, _, max, _) => min < max,
            _ => true,
        }
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

impl<T: Ord + FromStr> FromStr for Interval<T> {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref NULL_RE: Regex = Regex::new(r"^ *\( *Ø *\) *$").unwrap();
            static ref SINGLE_RE: Regex = Regex::new(r"^ *\[ *([^,]+) *\] *$").unwrap();
            static ref PAIR_RE: Regex =
                Regex::new(r"^ *(\[|\() *(.+?) *, *(.+?) *(\]|\)) *$").unwrap();
        }

        fn to_value<T: FromStr>(text: &str) -> Result<T, String> {
            text.parse::<T>()
                .map_err(|_err| String::from("Parse error!"))
        }

        if NULL_RE.is_match(s) {
            Ok(Interval::Empty)
        } else {
            match SINGLE_RE.captures(s) {
                Some(captures) => to_value(captures.get(1).unwrap().as_str()).map(Interval::Point),
                None => match PAIR_RE.captures(s) {
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
