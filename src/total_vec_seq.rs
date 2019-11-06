use alga::general::AbstractGroup;
use alga::general::AbstractLoop;
use alga::general::AbstractMagma;
use alga::general::AbstractMonoid;
use alga::general::AbstractQuasigroup;
use alga::general::AbstractSemigroup;
use alga::general::Additive;
use alga::general::ClosedAdd;
use alga::general::Identity;
use alga::general::TwoSidedInverse;
use std::cmp::max;
use std::cmp::min;
use std::fmt;
use std::fmt::Display;
use std::ops::{Add, AddAssign};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct TotalVecSeq<T> {
    default: T,
    values: Vec<T>,
}

impl<T> TotalVecSeq<T> {
    pub fn constant(value: T) -> Self {
        TotalVecSeq {
            default: value,
            values: Vec::new(),
        }
    }
}

impl<T: Eq> TotalVecSeq<T> {
    pub fn new(mut values: Vec<T>, default: T) -> TotalVecSeq<T> {
        let mut i = values.len();
        while i > 0 && values[i - 1] == default {
            i -= 1;
        }
        values.truncate(i);
        TotalVecSeq { default, values }
    }
    fn map<F: Fn(&T) -> T>(&self, op: F) -> Self {
        let default = op(&self.default);
        let mut values = Vec::with_capacity(self.values.len());
        for x in &self.values {
            values.push(op(x))
        }
        TotalVecSeq::new(values, default)
    }
    fn zip_with<F: Fn(&T, &T) -> T>(&self, rhs: &Self, op: F) -> Self {
        let lhs = self;
        let default = op(&lhs.default, &rhs.default);
        let mut values = Vec::with_capacity(max(lhs.values.len(), rhs.values.len()));
        let mut i = 0;
        while i < min(lhs.values.len(), rhs.values.len()) {
            values.push(op(&lhs.values[i], &rhs.values[i]));
            i += 1;
        }
        while i < lhs.values.len() {
            values.push(op(&lhs.values[i], &rhs.default));
            i += 1;
        }
        while i < rhs.values.len() {
            values.push(op(&lhs.default, &rhs.values[i]));
            i += 1;
        }
        TotalVecSeq::new(values, default)
    }
}

impl<T: Display> Display for TotalVecSeq<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TotalVecSeq([{}], {})",
            self.values
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(", "),
            self.default
        )
    }
}

impl<T: Identity<Additive>> Identity<Additive> for TotalVecSeq<T> {
    fn identity() -> Self {
        TotalVecSeq::constant(T::identity())
    }
}

impl<T: AbstractMagma<Additive> + Eq> AbstractMagma<Additive> for TotalVecSeq<T> {
    fn operate(&self, rhs: &Self) -> Self {
        TotalVecSeq::zip_with(self, rhs, T::operate)
    }
}

impl<T: TwoSidedInverse<Additive> + Eq> TwoSidedInverse<Additive> for TotalVecSeq<T> {
    fn two_sided_inverse(&self) -> Self {
        TotalVecSeq::map(self, T::two_sided_inverse)
    }
    fn two_sided_inverse_mut(&mut self) {
        self.default.two_sided_inverse_mut();
        for i in 0..self.values.len() {
            self.values[i].two_sided_inverse_mut();
        }
    }
}

impl<T: AbstractSemigroup<Additive> + Eq> AbstractSemigroup<Additive> for TotalVecSeq<T> {}
impl<T: AbstractMonoid<Additive> + Eq> AbstractMonoid<Additive> for TotalVecSeq<T> {}
impl<T: AbstractQuasigroup<Additive> + Eq> AbstractQuasigroup<Additive> for TotalVecSeq<T> {}
impl<T: AbstractLoop<Additive> + Eq> AbstractLoop<Additive> for TotalVecSeq<T> {}
impl<T: AbstractGroup<Additive> + Eq> AbstractGroup<Additive> for TotalVecSeq<T> {}
