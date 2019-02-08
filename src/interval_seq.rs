use super::*;
use interval::*;
use std::convert::From;
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::BitXor;
use std::ops::Not;
use std::slice::*;

impl<T: Ord + Eq> IntoIterator for IntervalSeq<T> {
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<T>;
    fn into_iter(self: IntervalSeq<T>) -> Self::IntoIter {
        self.values.into_iter()
    }
}

pub trait IntervalSet<T: Eq> {
    fn is_empty(&self) -> bool;
    fn is_contiguous(&self) -> bool;
    fn hull(&self) -> Interval<T>;
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
    K01 = 1,
    K10 = 2,
    K11 = 3,
}

impl BitAnd for Kind {
    type Output = Kind;
    fn bitand(self: Kind, that: Kind) -> Kind {
        Kind::from_u8((self as u8) & (that as u8))
    }
}

impl BitOr for Kind {
    type Output = Kind;
    fn bitor(self: Kind, that: Kind) -> Kind {
        Kind::from_u8((self as u8) | (that as u8))
    }
}

impl BitXor for Kind {
    type Output = Kind;
    fn bitxor(self: Kind, that: Kind) -> Kind {
        Kind::from_u8((self as u8) ^ (that as u8))
    }
}

impl Not for Kind {
    type Output = Kind;
    fn not(self: Kind) -> Kind {
        Kind::from_u8(!(self as u8))
    }
}

impl Kind {
    fn from_u8(value: u8) -> Kind {
        match value & 3 {
            0 => Kind::K00,
            1 => Kind::K01,
            2 => Kind::K10,
            3 => Kind::K11,
            _ => panic!(),
        }
    }
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

#[derive(Debug, Clone, PartialEq)]
pub struct IntervalSeq<T> {
    below_all: bool,
    values: Vec<T>,
    kinds: Vec<Kind>,
}

impl<T> From<bool> for IntervalSeq<T> {
    fn from(value: bool) -> IntervalSeq<T> {
        if value {
            IntervalSeq::all()
        } else {
            IntervalSeq::empty()
        }
    }
}

impl<T: Ord + Clone> From<Interval<T>> for IntervalSeq<T> {
    fn from(value: Interval<T>) -> IntervalSeq<T> {
        match (value.clone().lower_bound(), value.upper_bound()) {
            (Bound::Closed(ref a), Bound::Closed(ref b)) if a == b => IntervalSeq::at(a.clone()),
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

impl<T> IntervalSeq<T> {
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
    fn singleton(below_all: bool, value: T, kind: Kind) -> IntervalSeq<T> {
        IntervalSeq {
            below_all,
            values: vec![value],
            kinds: vec![kind],
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
}

impl<T: Ord + Clone> IntervalSeq<T> {
    fn from_to(from: T, fk: Kind, to: T, tk: Kind) -> IntervalSeq<T> {
        IntervalSeq {
            below_all: false,
            values: vec![from, to],
            kinds: vec![fk, tk],
        }
    }
}

pub struct Intervals<'a, T: Ord> {
    values: &'a Vec<T>,
    kinds: &'a Vec<Kind>,
    lower: Option<Bound<T>>,
    i: usize,
}

/**
 * Provide the abiltiy to read from an OpState
 */
trait Read<T: Ord> {
    fn a(&self) -> &IntervalSeq<T>;
    fn b(&self) -> &IntervalSeq<T>;
}

trait Builder<T: Ord> {
    fn copy_from(&mut self, src: &IntervalSeq<T>, i0: usize, i1: usize) -> ();
    fn flip_from(&mut self, src: &IntervalSeq<T>, i0: usize, i1: usize) -> ();
    fn append(&mut self, value: T, kind: Kind) -> ();
}

impl<T: Ord + Clone> Builder<T> for IntervalSeq<T> {
    fn copy_from(&mut self, src: &IntervalSeq<T>, i0: usize, i1: usize) -> () {
        self.values.extend_from_slice(&src.values[i0..i1]);
        self.kinds.extend_from_slice(&src.kinds[i0..i1]);
    }
    fn flip_from(&mut self, src: &IntervalSeq<T>, i0: usize, i1: usize) -> () {
        self.values.extend_from_slice(&src.values[i0..i1]);
        for i in i0..i1 {
            self.kinds.push(!src.kinds[i])
        }
    }
    fn append(&mut self, value: T, kind: Kind) -> () {
        let current = self.above_all();
        if (current && kind != Kind::K11) || (!current && kind != Kind::K00) {
            self.values.push(value);
            self.kinds.push(kind);
        }
    }
}

/**
 * State of an operation. Parametrized on the result type and the operation kind.
 */
struct OpState<T: Ord, R, K> {
    a: IntervalSeq<T>,
    b: IntervalSeq<T>,
    r: R,
    k: PhantomData<K>,
}

/**
 * Read impl for OpState
 */
impl<T: Ord, R, K> Read<T> for OpState<T, R, K> {
    fn a(&self) -> &IntervalSeq<T> {
        &self.a
    }
    fn b(&self) -> &IntervalSeq<T> {
        &self.b
    }
}

/**
 * Basic binary operation.
 */
trait BinaryOperation<T: Ord>: Read<T> {
    fn init(&mut self) -> ();
    fn from_a(&mut self, a0: usize, a1: usize, b: bool) -> ();
    fn from_b(&mut self, a: bool, b0: usize, b1: usize) -> ();
    fn collision(&mut self, ai: usize, bi: usize) -> ();
    fn merge0(&mut self, a0: usize, a1: usize, b0: usize, b1: usize) -> () {
        if a0 == a1 {
            self.from_b(self.a().below_index(a0), b0, b1)
        } else if b0 == b1 {
            self.from_a(a0, a1, self.b().below_index(b0))
        } else {
            let am: usize = (a0 + a1) / 2;
            match self.b().values[b0..b1].binary_search(&self.a().values[am]) {
                Result::Ok(i) => {
                    let bm = i + b0;
                    // same elements. bm is the index corresponding to am
                    // merge everything below a(am) with everything below the found element
                    self.merge0(a0, am, b0, bm);
                    // add the elements a(am) and b(bm)
                    self.collision(am, bm);
                    // merge everything above a(am) with everything above the found element
                    self.merge0(am + 1, a1, bm + 1, b1);
                }
                Result::Err(i) => {
                    let bi = i + b0;
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

struct AndOperation;

impl<T: Ord + Clone> BinaryOperation<T> for OpState<T, IntervalSeq<T>, AndOperation> {
    fn init(&mut self) -> () {
        self.r.below_all = self.a().below_all & self.b().below_all
    }
    fn from_a(&mut self, a0: usize, a1: usize, b: bool) -> () {
        if b {
            self.r.copy_from(&self.a, a0, a1)
        }
    }
    fn from_b(&mut self, a: bool, b0: usize, b1: usize) -> () {
        if a {
            self.r.copy_from(&self.b, b0, b1)
        }
    }
    fn collision(&mut self, ai: usize, bi: usize) -> () {
        let value = self.a.values[ai].clone();
        let kind = self.a.kinds[ai] & self.b.kinds[bi];
        self.r.append(value, kind)
    }
}

impl<T: Ord> Not for IntervalSeq<T> {
    type Output = IntervalSeq<T>;
    fn not(self: IntervalSeq<T>) -> IntervalSeq<T> {
        let mut kinds: Vec<Kind> = Vec::new();
        for x in self.kinds {
            kinds.push(!x);
        }
        IntervalSeq {
            below_all: !self.below_all,
            values: self.values,
            kinds: kinds,
        }
    }
}

impl<T: Ord + Clone> BitAnd for IntervalSeq<T> {
    type Output = IntervalSeq<T>;
    fn bitand(self: IntervalSeq<T>, that: IntervalSeq<T>) -> IntervalSeq<T> {
        let mut r: OpState<T, IntervalSeq<T>, AndOperation> = OpState {
            a: self,
            b: that,
            r: IntervalSeq::empty(),
            k: PhantomData {},
        };
        r.init();
        r.merge0(0, r.a.values.len(), 0, r.b.values.len());
        r.r
    }
}

struct OrOperation;

impl<T: Ord + Clone> BinaryOperation<T> for OpState<T, IntervalSeq<T>, OrOperation> {
    fn init(&mut self) -> () {
        self.r.below_all = self.a().below_all | self.b().below_all
    }
    fn from_a(&mut self, a0: usize, a1: usize, b: bool) -> () {
        if !b {
            self.r.copy_from(&self.a, a0, a1)
        }
    }
    fn from_b(&mut self, a: bool, b0: usize, b1: usize) -> () {
        if !a {
            self.r.copy_from(&self.b, b0, b1)
        }
    }
    fn collision(&mut self, ai: usize, bi: usize) -> () {
        let value = self.a.values[ai].clone();
        let kind = self.a.kinds[ai] | self.b.kinds[bi];
        self.r.append(value, kind)
    }
}

impl<T: Ord + Clone> BitOr for IntervalSeq<T> {
    type Output = IntervalSeq<T>;
    fn bitor(self: IntervalSeq<T>, that: IntervalSeq<T>) -> IntervalSeq<T> {
        let mut r: OpState<T, IntervalSeq<T>, OrOperation> = OpState {
            a: self,
            b: that,
            r: IntervalSeq::empty(),
            k: PhantomData {},
        };
        r.init();
        r.merge0(0, r.a.values.len(), 0, r.b.values.len());
        r.r
    }
}

struct XorOperation;

impl<T: Ord + Clone> BinaryOperation<T> for OpState<T, IntervalSeq<T>, XorOperation> {
    fn init(&mut self) -> () {
        self.r.below_all = self.a().below_all ^ self.b().below_all
    }
    fn from_a(&mut self, a0: usize, a1: usize, b: bool) -> () {
        if !b {
            self.r.copy_from(&self.a, a0, a1)
        } else {
            self.r.flip_from(&self.a, a0, a1)
        }
    }
    fn from_b(&mut self, a: bool, b0: usize, b1: usize) -> () {
        if !a {
            self.r.copy_from(&self.b, b0, b1)
        } else {
            self.r.flip_from(&self.b, b0, b1)
        }
    }
    fn collision(&mut self, ai: usize, bi: usize) -> () {
        let value = self.a.values[ai].clone();
        let kind = self.a.kinds[ai] ^ self.b.kinds[bi];
        self.r.append(value, kind)
    }
}

impl<T: Ord + Clone> BitXor for IntervalSeq<T> {
    type Output = IntervalSeq<T>;
    fn bitxor(self: IntervalSeq<T>, that: IntervalSeq<T>) -> IntervalSeq<T> {
        let mut r: OpState<T, IntervalSeq<T>, XorOperation> = OpState {
            a: self,
            b: that,
            r: IntervalSeq::empty(),
            k: PhantomData {},
        };
        r.init();
        r.merge0(0, r.a.values.len(), 0, r.b.values.len());
        r.r
    }
}

impl<T: Ord + FromStr + Clone> FromStr for IntervalSeq<T> {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            return Result::Ok(IntervalSeq::empty());
        }
        let intervals: Result<Vec<Interval<T>>, _> =
            s.split(";").map(|x| x.parse::<Interval<T>>()).collect();
        intervals
            .map(|is| {
                is.iter()
                    .map(|x| IntervalSeq::from(x.clone()))
                    .fold(IntervalSeq::empty(), BitOr::bitor)
            })
            .map_err(|_| String::from("Parse error"))
    }
}

impl<'a, T: Ord + Clone> Intervals<'a, T> {
    fn next_interval(self: &mut Intervals<'a, T>) -> Option<Interval<T>> {
        if self.i < self.values.len() {
            let value = self.values[self.i].clone();
            let kind = self.kinds[self.i];
            self.i += 1;
            match (self.lower.clone(), kind) {
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
                (Option::None, _) => {
                    panic!();
                }

                (Option::Some(lower), Kind::K01) => {
                    let upper = Bound::Open(value);
                    self.lower = Some(upper.clone());
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
            match self.lower.clone() {
                Some(lower) => {
                    self.lower = None;
                    Some(Interval::from_bounds(lower, Bound::Unbound))
                }
                None => None,
            }
        }
    }
}

impl<'a, T: Ord + Clone> std::iter::Iterator for Intervals<'a, T> {
    type Item = Interval<T>;

    fn next(self: &mut Intervals<'a, T>) -> Option<Interval<T>> {
        let has_next = self.i < self.values.len() || self.lower.is_some();
        match Intervals::next_interval(self) {
            Some(x) => Some(x),
            None if has_next => Intervals::next(self),
            _ => None,
        }
    }
}

impl<T: Ord> IntervalSeq<T> {
    #[cfg(test)]
    pub fn is_valid(&self) -> bool {
        if self.values.len() != self.kinds.len() {
            // kinds and values not of the same size
            false
        } else {
            let mut prev = self.below_all;
            for i in 0..self.values.len() {
                if i > 0 && self.values[i - 1] >= self.values[i] {
                    // values unordered
                    return false;
                }
                let k = self.kinds[i];
                if (!prev && k == Kind::K00) || (prev && k == Kind::K11) {
                    // redundant element
                    return false;
                }
                prev = k.value_above();
            }
            true
        }
    }
    fn below_index(self: &IntervalSeq<T>, index: usize) -> bool {
        if index == 0 {
            self.below_all
        } else {
            self.kinds[index - 1].value_above()
        }
    }
}

impl<T: Ord + Clone> IntervalSeq<T> {
    pub fn intervals(self: &IntervalSeq<T>) -> Intervals<T> {
        Intervals {
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

    pub fn edges(self: &IntervalSeq<T>) -> Edges<T> {
        Edges(self.values.iter())
    }
}

pub struct Edges<'a, T>(std::slice::Iter<'a, T>);

impl<'a, T: Ord> std::iter::Iterator for Edges<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        let Edges(inner) = self;
        inner.next()
    }
}

impl<T: Ord + Clone> IntervalSeq<T> {
    fn lower_bound(&self, i: usize) -> Bound<T> {
        match self.kinds[i] {
            Kind::K01 => Bound::Open(self.values[i].clone()),
            Kind::K11 => Bound::Closed(self.values[i].clone()),
            Kind::K10 => Bound::Closed(self.values[i].clone()),
            _ => panic!(),
        }
    }

    fn upper_bound(&self, i: usize) -> Bound<T> {
        match self.kinds[i] {
            Kind::K10 => Bound::Closed(self.values[i].clone()),
            Kind::K00 => Bound::Open(self.values[i].clone()),
            _ => panic!(),
        }
    }
}

impl<T: Ord + Clone + Display> Display for IntervalSeq<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text: String = self
            .intervals()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>()
            .join("; ");
        write!(f, "{}", text)
    }
}

impl<T: Ord + Clone> IntervalSet<T> for IntervalSeq<T> {
    fn hull(self: &IntervalSeq<T>) -> Interval<T> {
        if self.is_empty() {
            Interval::Empty
        } else if self.below_all() && self.above_all() {
            Interval::All
        } else if self.below_all() {
            Interval::from_bounds(Bound::Unbound, self.upper_bound(self.values.len() - 1))
        } else if self.above_all() {
            Interval::from_bounds(self.lower_bound(0), Bound::Unbound)
        } else {
            Interval::from_bounds(self.lower_bound(0), self.upper_bound(self.values.len() - 1))
        }
    }

    fn is_contiguous(self: &IntervalSeq<T>) -> bool {
        if self.below_all {
            if self.is_empty() {
                true
            } else {
                self.kinds.len() == 1 && self.kinds[0] != Kind::K01
            }
        } else {
            if self.is_empty() {
                true
            } else if self.kinds.len() == 1 {
                true
            } else if self.kinds.len() == 2 {
                let a = self.kinds[0];
                let b = self.kinds[1];
                // a: K01 or K11, K00 redundant, K10 not cont
                (a == Kind::K01 || a == Kind::K11) &&
                // b: K10 or K00, K11 redundant, K01 not cont
                (b == Kind::K10 || b == Kind::K00)
            } else {
                false
            }
        }
    }

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
            .map_or(self.below_all, |k| k.value_above())
    }
}
