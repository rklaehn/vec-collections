use super::*;
use alga::general::AbstractMagma;
use alga::general::AbstractMonoid;
use alga::general::AbstractSemigroup;
use alga::general::Additive;
use alga::general::AdditiveSemigroup;
use alga::general::Identity;
use std::fmt;
use std::fmt::Display;
use std::ops::{Add, AddAssign};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct ArraySeq<T>(Vec<T>);

impl<T: Eq> ArraySeq<T> {
    fn as_total(self, default: T) -> TotalArraySeq<T> {
        TotalArraySeq::new(self.0, default)
    }
}

impl<T> From<Vec<T>> for ArraySeq<T> {
    fn from(value: Vec<T>) -> Self {
        ArraySeq(value)
    }
}

impl<T> Into<Vec<T>> for ArraySeq<T> {
    fn into(self) -> Vec<T> {
        self.0
    }
}

impl<T: Display> Display for ArraySeq<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ArraySeq([{}])",
            self.0
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl<T> Identity<Additive> for ArraySeq<T> {
    fn identity() -> Self {
        ArraySeq(Vec::new())
    }
}

impl<T: Clone> AbstractMagma<Additive> for ArraySeq<T> {
    fn operate(&self, right: &Self) -> Self {
        let mut res = self.0.clone();
        res.extend(right.0.clone());
        ArraySeq(res)
    }
}

impl<T: Clone + Eq> AbstractSemigroup<Additive> for ArraySeq<T> {}
impl<T: Clone + Eq> AbstractMonoid<Additive> for ArraySeq<T> {}
impl<'a, 'b, T: Clone + Eq> Add<&'b ArraySeq<T>> for &'a ArraySeq<T> {
    type Output = ArraySeq<T>;
    fn add(self, rhs: &'b ArraySeq<T>) -> ArraySeq<T> {
        self.operate(rhs)
    }
}
impl<T: Clone + Eq> AddAssign<&ArraySeq<T>> for ArraySeq<T> {
    fn add_assign(&mut self, rhs: &ArraySeq<T>) {
        *self = self.operate(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_usage() {
        let a: ArraySeq<i64> = vec![1, 2, 3].into();
        let b: ArraySeq<i64> = vec![1, 2, 3].into();
        let mut c = &a + &b;
        c += &b;
        let at = a.clone().as_total(0);
        let bt = b.clone().as_total(0);
        let ct = at.operate(&bt);
        println!("{} {} {}", a, b, c);
        println!("{} {} {}", at, bt, ct);
    }
}
