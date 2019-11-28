use super::*;
use alga::general::AbstractMagma;
use alga::general::AbstractMonoid;
use alga::general::AbstractSemigroup;
use alga::general::Additive;
use alga::general::Identity;
use std::fmt;
use std::fmt::Display;
use std::ops::{Add, AddAssign};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct VecSeq<T>(Vec<T>);

impl<T: Eq> VecSeq<T> {
    pub fn into_total(self, default: T) -> TotalVecSeq<T> {
        TotalVecSeq::new(self.0, default)
    }
}

impl<T> From<Vec<T>> for VecSeq<T> {
    fn from(value: Vec<T>) -> Self {
        VecSeq(value)
    }
}

impl<T> Into<Vec<T>> for VecSeq<T> {
    fn into(self) -> Vec<T> {
        self.0
    }
}

impl<T: Display> Display for VecSeq<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "VecSeq([{}])",
            self.0
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl<T> Identity<Additive> for VecSeq<T> {
    fn identity() -> Self {
        VecSeq(Vec::new())
    }
}

impl<T: Clone> AbstractMagma<Additive> for VecSeq<T> {
    fn operate(&self, right: &Self) -> Self {
        let mut res = self.0.clone();
        res.extend(right.0.clone());
        VecSeq(res)
    }
}

impl<T: Clone + Eq> AbstractSemigroup<Additive> for VecSeq<T> {}
impl<T: Clone + Eq> AbstractMonoid<Additive> for VecSeq<T> {}
impl<'a, 'b, T: Clone + Eq> Add<&'b VecSeq<T>> for &'a VecSeq<T> {
    type Output = VecSeq<T>;
    fn add(self, rhs: &'b VecSeq<T>) -> VecSeq<T> {
        self.operate(rhs)
    }
}
impl<T: Clone + Eq> AddAssign<&VecSeq<T>> for VecSeq<T> {
    fn add_assign(&mut self, rhs: &VecSeq<T>) {
        *self = self.operate(rhs)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_basic_usage() {
//         let a: VecSeq<i64> = vec![1, 2, 3].into();
//         let b: VecSeq<i64> = vec![1, 2, 3].into();
//         let mut c = &a + &b;
//         c += &b;
//         let at = a.clone().into_total(0);
//         let bt = b.clone().into_total(0);
//         let ct = at.operate(&bt);
//         println!("{} {} {}", a, b, c);
//         println!("{} {} {}", at, bt, ct);
//     }
// }
