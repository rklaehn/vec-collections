use num_traits::{Bounded, One, Zero};
use std::{
    cmp::max,
    fmt,
    fmt::Display,
    ops::{Add, Div, Index, Mul, Neg, Sub},
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct TotalVecSeq<T> {
    default: T,
    values: Vec<T>,
}

impl<T: Add<Output = T> + Eq + Clone> Add for TotalVecSeq<T> {
    type Output = Self;

    fn add(self, that: Self) -> Self::Output {
        self.combine(that, |a, b| a + b)
    }
}

impl<T: Sub<Output = T> + Eq + Clone> Sub for TotalVecSeq<T> {
    type Output = Self;

    fn sub(self, that: Self) -> Self::Output {
        self.combine(that, |a, b| a - b)
    }
}

impl<T: Neg<Output = T> + Eq + Clone> Neg for TotalVecSeq<T> {
    type Output = Self;
    fn neg(self) -> Self {
        self.map(|x| -x)
    }
}

impl<T: Mul<Output = T> + Eq + Clone> Mul for TotalVecSeq<T> {
    type Output = Self;

    fn mul(self, that: Self) -> Self {
        self.combine(that, |a, b| a * b)
    }
}

impl<T: Div<Output = T> + Eq + Clone> Div for TotalVecSeq<T> {
    type Output = Self;

    fn div(self, that: Self) -> Self {
        self.combine(that, |a, b| a / b)
    }
}

impl<T: Bounded> Bounded for TotalVecSeq<T> {
    fn min_value() -> Self {
        T::min_value().into()
    }
    fn max_value() -> Self {
        T::max_value().into()
    }
}

impl<T: Zero + Eq + Clone> Zero for TotalVecSeq<T> {
    fn zero() -> Self {
        T::zero().into()
    }
    fn is_zero(&self) -> bool {
        self.values.is_empty() && self.default.is_zero()
    }
}

impl<T: One + Eq + Clone> One for TotalVecSeq<T> {
    fn one() -> Self {
        T::one().into()
    }
    fn is_one(&self) -> bool {
        self.values.is_empty() && self.default.is_one()
    }
}

impl<T> From<T> for TotalVecSeq<T> {
    fn from(value: T) -> Self {
        Self::constant(value)
    }
}

impl<T> TotalVecSeq<T> {
    pub fn constant(value: T) -> Self {
        TotalVecSeq {
            default: value,
            values: Vec::new(),
        }
    }
}

impl<V> Index<usize> for TotalVecSeq<V> {
    type Output = V;
    fn index(&self, index: usize) -> &V {
        self.values.get(index).unwrap_or(&self.default)
    }
}

impl<T: Eq> TotalVecSeq<T> {
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.values.iter().chain(std::iter::repeat(&self.default))
    }

    fn combine_ref<F: Fn(&T, &T) -> T>(&self, rhs: &Self, op: F) -> Self {
        let lhs = self;
        let l = max(lhs.values.len(), rhs.values.len());
        let default = op(&lhs.default, &rhs.default);
        let values: Vec<T> = lhs
            .iter()
            .zip(rhs.iter())
            .map(|(a, b)| op(a, b))
            .take(l)
            .collect();
        TotalVecSeq::new(values, default)
    }

    #[allow(dead_code)]
    fn map_ref<F: Fn(&T) -> T>(&self, op: F) -> Self {
        let default = op(&self.default);
        let mut values = Vec::with_capacity(self.values.len());
        for x in &self.values {
            values.push(op(x))
        }
        TotalVecSeq::new(values, default)
    }
}

impl<T: Eq + Ord + Clone> TotalVecSeq<T> {
    pub fn supremum(&self, that: &Self) -> Self {
        self.combine_ref(that, |a, b| if a > b { a.clone() } else { b.clone() })
    }
    pub fn infimum(&self, that: &Self) -> Self {
        self.combine_ref(that, |a, b| if a < b { a.clone() } else { b.clone() })
    }
}

impl<T: Eq + Clone> TotalVecSeq<T> {
    fn into_iter(self) -> impl Iterator<Item = T> {
        self.values
            .into_iter()
            .chain(std::iter::repeat(self.default))
    }

    fn combine<F: Fn(T, T) -> T>(self, rhs: Self, op: F) -> Self {
        let lhs = self;
        let l = max(lhs.values.len(), rhs.values.len());
        let default = op(lhs.default.clone(), rhs.default.clone());
        let values: Vec<T> = lhs
            .into_iter()
            .zip(rhs.into_iter())
            .map(|(a, b)| op(a, b))
            .take(l)
            .collect();
        TotalVecSeq::new(values, default)
    }

    fn map<F: Fn(T) -> T>(self, op: F) -> Self {
        let default = op(self.default.clone());
        let values: Vec<_> = self.into_iter().map(op).collect();
        TotalVecSeq::new(values, default)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage() {
        type Test = TotalVecSeq<i64>;
        let x: Test = Test::new(vec![1, 2, 3], 0);
        let y: Test = Test::new(vec![4, 5], 6);
        let z: Test = x + y;
        println!("{}", z);
    }
}

// impl<T: Identity<Additive>> Identity<Additive> for TotalVecSeq<T> {
//     fn identity() -> Self {
//         TotalVecSeq::constant(T::identity())
//     }
// }

// impl<T: AbstractMagma<Additive> + Eq> AbstractMagma<Additive> for TotalVecSeq<T> {
//     fn operate(&self, rhs: &Self) -> Self {
//         TotalVecSeq::zip_with(self, rhs, T::operate)
//     }
// }

// impl<T: TwoSidedInverse<Additive> + Eq> TwoSidedInverse<Additive> for TotalVecSeq<T> {
//     fn two_sided_inverse(&self) -> Self {
//         TotalVecSeq::map(self, T::two_sided_inverse)
//     }
//     fn two_sided_inverse_mut(&mut self) {
//         self.default.two_sided_inverse_mut();
//         for i in 0..self.values.len() {
//             self.values[i].two_sided_inverse_mut();
//         }
//     }
// }

// impl<T: AbstractSemigroup<Additive> + Eq> AbstractSemigroup<Additive> for TotalVecSeq<T> {}
// impl<T: AbstractMonoid<Additive> + Eq> AbstractMonoid<Additive> for TotalVecSeq<T> {}
// impl<T: AbstractQuasigroup<Additive> + Eq> AbstractQuasigroup<Additive> for TotalVecSeq<T> {}
// impl<T: AbstractLoop<Additive> + Eq> AbstractLoop<Additive> for TotalVecSeq<T> {}
// impl<T: AbstractGroup<Additive> + Eq> AbstractGroup<Additive> for TotalVecSeq<T> {}
