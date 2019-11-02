use crate::ArraySet;
use std::fmt::Debug;

pub enum TotalArraySet<T> {
    elements: ArraySet<T>,
    negated: bool,
}

impl<T> TotalArraySet<T> {

    fn new(elements: ArraySet<T>, negated: bool) -> Self {
        Self {
            elements, negated,
        }
    }
    fn is_empty(&self) -> bool {
        !self.negated && self.elements.is_empty()
    }

    fn negate(self) -> Self {
        TotalArraySet {
            elements: self.elements,
            negated: !self.negated,
        }
    }
}

impl<T: Ord + Default + Copy + Debug> TotalArraySet<T> {

    pub fn is_subset(&self, that: &Self) -> bool {
        match (self.negated, that.negated) {
            (false, false) => self.elements.is_subset(&that.elements),
            (false, true) => unimplemented!(),
            (true, false) => false,
            (true, true) => self.elements.is_subset(&that.elements),
        }
    }

    pub fn is_disjoint(&self, that: &Self) -> bool {
        match (self.negated, that.negated) {
            (false, false) => self.elements.is_disjoint(&that.elements),
            (false, true) => self.elements.is_subset(&that.elements),
            (true, false) => self.elements.is_superset(&that.elements),
            (true, true) => false,
        }
    }

    pub fn xor(self, that: Self) -> Self {
        Self::new(
            self.elements ^ that.elements,
            self.negated ^ that.negated,
        )
    }

    pub fn union(self, that: Self) -> Self {
        match (self.negated, that.negated) {
            // union of elements
            (false, false) => Self::new(self.elements | that.elements, false),
            // remove holes from that
            (false, true) => Self::new(that.elements - self.elements, true),
            // remove holes from self
            (true, false) => Self::new(self.elements - that.elements, true),
            // intersection of holes
            (true, true) => Self::new(that.elements & self.elements, true),
        }
    }

    pub fn intersection(self, that: Self) -> Self {
        match (self.negated, that.negated) {
            // intersection of elements
            (false, false) => Self::new(self.elements & that.elements, false),
            // remove elements from self
            (false, true) => Self::new(self.elements - that.elements, false),
            // remove elements from that
            (true, false) => Self::new(that.elements - self.elements, false),
            // union of elements
            (true, true) => Self::new(that.elements | self.elements, true),
        }
    }

    pub fn difference(self, that: Self) -> Self {
        match (self.negated, that.negated) {
            // intersection of elements
            (false, false) => Self::new(self.elements - that.elements, false),
            // keep only holes of that
            (false, true) => Self::new(self.elements & that.elements, false),
            // add holes from that
            (true, false) => Self::new(self.elements | that.elements, true),
            // union of elements
            (true, true) => Self::new(that.elements - self.elements, true),
        }
    }
}
