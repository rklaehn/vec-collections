use super::*;
use core::ops::Neg;
use std::fmt;
use std::fmt::Display;
use std::slice::*;

impl<T: Ord + Eq> IntoIterator for IntervalSeq<T> {
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<T>;
    fn into_iter(self: IntervalSeq<T>) -> Self::IntoIter {
        self.values.into_iter()
    }
}

trait IntervalSet<T: Eq> {
    fn is_empty(&self) -> bool;
    // fn is_contiguous(&self) -> bool;
    // fn hull(&self) -> Interval<T>;
    fn at(&self, value: T) -> bool;
    fn above(&self, value: T) -> bool;
    fn below(&self, value: T) -> bool;
    fn above_all(&self) -> bool;
    fn below_all(&self) -> bool;
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
enum Kind {
    K00 = 0,
    K10 = 1,
    K01 = 2,
    K11 = 3,
}

impl Neg for Kind {
    type Output = Kind;
    fn neg(self: Kind) -> Kind {
        match self {
            Kind::K00 => Kind::K11,
            Kind::K10 => Kind::K01,
            Kind::K01 => Kind::K10,
            Kind::K11 => Kind::K00,
        }
    }
}

impl Kind {
    fn value_at(self: &Kind) -> bool {
        match self {
            Kind::K10 | Kind::K11 => true,
            _ => false,
        }
    }

    fn value_above(self: &Kind) -> bool {
        match self {
            Kind::K01 | Kind::K11 => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct IntervalSeq<T: Eq + Ord> {
    below_all: bool,
    values: Vec<T>,
    kinds: Vec<Kind>,
}

impl<T: Ord + Copy> IntervalSeq<T> {
    fn singleton(below_all: bool, value: T, kind: Kind) -> IntervalSeq<T> {
        IntervalSeq {
            below_all,
            values: vec![value],
            kinds: vec![kind],
        }
    }
    fn from_to(from: T, fk: Kind, to: T, tk: Kind) -> IntervalSeq<T> {
        IntervalSeq {
            below_all: false,
            values: vec![from, to],
            kinds: vec![fk, tk],
        }
    }
    pub fn empty() -> IntervalSeq<T> {
        IntervalSeq {
            below_all: false,
            kinds: Vec::new(),
            values: Vec::new(),
        }
    }
    pub fn all() -> IntervalSeq<T> {
        IntervalSeq {
            below_all: true,
            kinds: Vec::new(),
            values: Vec::new(),
        }
    }
    pub fn at(value: T) -> IntervalSeq<T> {
        IntervalSeq::singleton(false, value, Kind::K10)
    }
    pub fn except(value: T) -> IntervalSeq<T> {
        IntervalSeq::singleton(true, value, Kind::K01)
    }
    pub fn below(value: T) -> IntervalSeq<T> {
        IntervalSeq::singleton(true, value, Kind::K00)
    }
    pub fn at_or_below(value: T) -> IntervalSeq<T> {
        IntervalSeq::singleton(true, value, Kind::K10)
    }
    pub fn at_or_above(value: T) -> IntervalSeq<T> {
        IntervalSeq::singleton(false, value, Kind::K11)
    }
    pub fn above(value: T) -> IntervalSeq<T> {
        IntervalSeq::singleton(false, value, Kind::K01)
    }
    pub fn from_interval(value: Interval<T>) -> IntervalSeq<T> {
        match (value.lower_bound(), value.upper_bound()) {
            (Bound::Closed(a), Bound::Closed(b)) if a == b => IntervalSeq::at(a),
            (Bound::Unbound, Bound::Open(x)) => IntervalSeq::below(x),
            (Bound::Unbound, Bound::Closed(x)) => IntervalSeq::at_or_below(x),
            (Bound::Open(x), Bound::Unbound) => IntervalSeq::above(x),
            (Bound::Closed(x), Bound::Unbound) => IntervalSeq::at_or_above(x),
            (Bound::Closed(a), Bound::Closed(b)) => {
                IntervalSeq::from_to(a, Kind::K11, b, Kind::K10)
            }
            (Bound::Closed(a), Bound::Open(b)) => IntervalSeq::from_to(a, Kind::K11, b, Kind::K00),
            (Bound::Open(a), Bound::Closed(b)) => IntervalSeq::from_to(a, Kind::K01, b, Kind::K10),
            (Bound::Open(a), Bound::Open(b)) => IntervalSeq::from_to(a, Kind::K01, b, Kind::K00),
            (Bound::Unbound, Bound::Unbound) => IntervalSeq::all(),
            (Bound::Empty, Bound::Empty) => IntervalSeq::empty(),
            _ => IntervalSeq::empty(),
        }
    }
}

struct IntervalIterator<'a, T: Ord> {
    values: &'a Vec<T>,
    kinds: &'a Vec<Kind>,
    lower: Option<Bound<T>>,
    i: usize,
}

trait MergeOps<T: Ord> {
    fn a(&self) -> IntervalSeq<T>;
    fn b(&self) -> IntervalSeq<T>;
    fn from_a(&mut self, a0: usize, a1: usize, b: bool) -> ();
    fn from_b(&mut self, a: bool, b0: usize, b1: usize) -> ();
    fn op(&mut self, a: bool, b: bool) -> ();
    fn collision(&mut self, ai: usize, bi: usize) -> ();
    fn merge0(&mut self, a0: usize, a1: usize, b0: usize, b1: usize) -> () {
        if a0 == a1 {
            self.from_b(self.a().below_index(a0), b0, b1)
        } else if b0 == b1 {
            self.from_a(a0, a1, self.b().below_index(b0))
        } else {
            let am: usize = (a0 + a1) / 2;
            match self.b().values[b0..b1].binary_search(&self.a().values[am]) {
                Result::Ok(bm) => {
                    // same elements. bm is the index corresponding to am
                    // merge everything below a(am) with everything below the found element
                    self.merge0(a0, am, b0, bm);
                    // add the elements a(am) and b(bm)
                    self.collision(am, bm);
                    // merge everything above a(am) with everything above the found element
                    self.merge0(am + 1, a1, bm + 1, b1);
                }
                Result::Err(bi) => {
                    // not found. bi is the insertion point
                    // merge everything below a(am) with everything below the found insertion point
                    self.merge0(a0, am, b0, bi);
                    // add a(am)
                    self.from_a(am, am + 1, self.b().below_index(bi));
                    // everything above a(am) with everything above the found insertion point
                    self.merge0(am + 1, a1, bi, b1);
                }
            }
        }
    }
}

struct AndMerger<T: Ord> {
    a: IntervalSeq<T>,
    b: IntervalSeq<T>,
}

impl<T: Ord + FromStr + Display + Copy> FromStr for IntervalSeq<T> {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _intervals: Result<Vec<Interval<T>>, _> =
            s.split(";").map(|x| x.parse::<Interval<T>>()).collect();
        Ok(IntervalSeq::empty())
    }
}

impl<'a, T: Ord + Copy> IntervalIterator<'a, T> {
    fn next_interval(self: &mut IntervalIterator<'a, T>) -> Option<Interval<T>> {
        if self.i < self.values.len() {
            let value = self.values[self.i];
            let kind = self.kinds[self.i];
            self.i += 1;
            match (self.lower, kind) {
                (Option::None, Kind::K10) => {
                    self.lower = None;
                    Some(Interval::Point(value))
                }
                (Option::None, Kind::K11) => {
                    self.lower = Some(Bound::Closed(value));
                    None
                }
                (Option::None, Kind::K01) => {
                    self.lower = Some(Bound::Open(value));
                    None
                }
                (Option::None, _) => panic!(),

                (Option::Some(lower), Kind::K01) => {
                    let upper = Bound::Open(value);
                    self.lower = Some(upper);
                    Some(Interval::from_bounds(lower, upper))
                }
                (Option::Some(lower), Kind::K00) => {
                    let upper = Bound::Open(value);
                    self.lower = None;
                    Some(Interval::from_bounds(lower, upper))
                }
                (Option::Some(lower), Kind::K10) => {
                    let upper = Bound::Closed(value);
                    self.lower = None;
                    Some(Interval::from_bounds(lower, upper))
                }
                (Option::Some(_), _) => {
                    panic!();
                }
            }
        } else {
            match self.lower {
                Some(lower) => {
                    self.lower = None;
                    Some(Interval::from_bounds(lower, Bound::Unbound))
                }
                None => None,
            }
        }
    }
}

impl<'a, T: Ord + Copy> std::iter::Iterator for IntervalIterator<'a, T> {
    type Item = Interval<T>;

    fn next(self: &mut IntervalIterator<'a, T>) -> Option<Interval<T>> {
        let has_next = self.i < self.values.len() || self.lower.is_some();
        match IntervalIterator::next_interval(self) {
            Some(x) => Some(x),
            None if has_next => IntervalIterator::next(self),
            _ => None,
        }
    }
}

impl<T: Ord> IntervalSeq<T> {
    fn below_index(self: &IntervalSeq<T>, index: usize) -> bool {
        if index == 0 {
            self.below_all
        } else {
            self.kinds[index - 1].value_above()
        }
    }

    fn intervals(self: &IntervalSeq<T>) -> IntervalIterator<T> {
        IntervalIterator {
            i: 0,
            kinds: &self.kinds,
            values: &self.values,
            lower: if self.below_all {
                Some(Bound::Unbound)
            } else {
                None
            },
        }
    }

    fn edges(self: IntervalSeq<T>) -> Vec<T> {
        self.values
    }
}

impl<T: Eq + Ord + Copy + Display> Display for IntervalSeq<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text: String = self
            .intervals()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>()
            .join("; ");
        write!(f, "{}", text)
    }
}

impl<T: Eq + Ord> IntervalSet<T> for IntervalSeq<T> {
    fn is_empty(self: &IntervalSeq<T>) -> bool {
        !self.below_all && self.values.is_empty()
    }

    fn at(self: &IntervalSeq<T>, value: T) -> bool {
        match self.values.as_slice().binary_search(&value) {
            Ok(index) => self.kinds[index].value_at(),
            Err(index) => self.below_index(index),
        }
    }
    fn above(self: &IntervalSeq<T>, value: T) -> bool {
        match self.values.as_slice().binary_search(&value) {
            Ok(index) => self.kinds[index].value_above(),
            Err(index) => self.below_index(index),
        }
    }
    fn below(self: &IntervalSeq<T>, value: T) -> bool {
        match self.values.as_slice().binary_search(&value) {
            Ok(index) => {
                if index > 0 {
                    self.kinds[index - 1].value_above()
                } else {
                    self.below_all
                }
            }
            Err(index) => self.below_index(index),
        }
    }
    fn below_all(self: &IntervalSeq<T>) -> bool {
        self.below_all
    }
    fn above_all(self: &IntervalSeq<T>) -> bool {
        self.kinds
            .last()
            .map_or(self.below_all(), |k| k.value_above())
    }
}
