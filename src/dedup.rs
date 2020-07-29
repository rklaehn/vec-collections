use std::{marker::PhantomData, cmp::{min, Ordering}, ops::DerefMut};
use smallvec::SmallVec;

/// deduplicate a slice, moving the duplicates to the end.
/// returns the number of unique elements.
///
/// there is an unstable library feature called slice.partition_dedup which is
/// roughly similar: https://github.com/rust-lang/rust/issues/54279
///
/// the library feature would be preferable since it is unsafe and thus has no bounds checks.
pub fn dedup_by<T, F: Fn(&T, &T) -> bool>(d: &mut [T], same_bucket: F, keep: Keep) -> usize {
    if d.is_empty() {
        return 0;
    }
    let mut j = 0;
    for i in 1..d.len() {
        if !same_bucket(&d[i], &d[j]) {
            j += 1;
            if i != j {
                d.swap(i, j);
            }
        } else if keep == Keep::Last {
            d.swap(i, j);
        }
    }
    j + 1
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Keep {
    First,
    Last,
}

trait Inner<T>: DerefMut<Target=[T]> {
    fn push(&mut self, value: T);
    fn truncate(&mut self, size: usize);
}

impl<T> Inner<T> for Vec<T> {
    fn push(&mut self, value: T) {
        self.push(value)
    }
    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}

impl<A: smallvec::Array> Inner<A::Item> for SmallVec<A> {
    fn push(&mut self, value: A::Item) {
        self.push(value)
    }
    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}

/// an aggregator to incrementally sort and deduplicate unsorted elements
///
/// this is a compromise between sorting and deduping at the end, which can have a lot of
/// temporary memory usage if you feed it with lots of duplicate elements, and sorting
/// on each insertion, which is expensive for a flat data structure due to all the memory
/// movements.
struct SortAndDedup<I, T, F> {
    /// partially sorted and deduplicated data elements
    data: I,
    /// number of sorted elements
    sorted: usize,
    /// comparison
    cmp: F,
    /// total number of unsorted elements that have been added
    count: usize,
    /// which element to keep in case of duplicates
    keep: Keep,

    _t: PhantomData<T>,
}

pub fn sort_and_dedup<I: Iterator>(iter: I) -> Vec<I::Item>
    where I::Item: Ord
{
    let mut agg: SortAndDedup<Vec<I::Item>, I::Item, _> = SortAndDedup {
        data: Vec::with_capacity(iter.size_hint().0),
        count: 0,
        sorted: 0,
        cmp: |a: &I::Item, b: &I::Item| a.cmp(b),
        keep: Keep::First,
        _t: PhantomData,
    };
    for x in iter {
        agg.push(x);
    }
    agg.into_inner()
}

pub fn sort_and_dedup_by_key<K, I, F>(iter: I, key: F, keep: Keep) -> Vec<I::Item>
where
    K: Ord,
    I: Iterator,
    F: Fn(&I::Item) -> &K,
{
    let mut agg: SortAndDedup<Vec<I::Item>, I::Item, _> = SortAndDedup {
        data: Vec::with_capacity(min(iter.size_hint().0, 16)),
        count: 0,
        sorted: 0,
        cmp: |a: &I::Item, b: &I::Item| key(a).cmp(&key(b)),
        keep,
        _t: PhantomData,
    };
    for x in iter {
        agg.push(x);
    }
    agg.into_inner()
}

impl<I, T, F> SortAndDedup<I, T, F>
where
    F: Fn(&T, &T) -> Ordering,
    I: Inner<T>,
{
    fn sort_and_dedup(&mut self) {
        if self.sorted < self.data.len() {
            let cmp = &self.cmp;
            let slice = self.data.deref_mut();
            // this must be a stable sort for the keep feature to work
            // since typically the first 50% are already sorted, we benefit from a sort algo that optimizes for that, like timsort
            slice.sort_by(cmp);
            let unique = dedup_by(slice, |a, b| cmp(a, b) == Ordering::Equal, self.keep);
            self.data.truncate(unique);
            self.sorted = self.data.len();
        }
    }

    fn into_inner(self) -> I {
        let mut res = self;
        res.sort_and_dedup();
        res.data
    }

    fn push(&mut self, elem: T) {
        self.count += 1;
        if self.sorted == self.data.len() {
            if let Some(last) = self.data.last_mut() {
                match (self.cmp)(last, &elem) {
                    Ordering::Less => {
                        // remain sorted
                        self.sorted += 1;
                        self.data.push(elem);
                    }
                    Ordering::Equal => {
                        // remain sorted, just replace the end if needed
                        if self.keep == Keep::Last {
                            *last = elem;
                        }
                    }
                    Ordering::Greater => {
                        // unsorted
                        self.data.push(elem);
                    }
                }
            } else {
                // single element is always sorted
                self.sorted += 1;
                self.data.push(elem);
            }
        } else {
            // not sorted
            self.data.push(elem);
        }
        // Don't bother with the compaction for small collections
        if self.data.len() >= 16 {
            let sorted = self.sorted;
            let unsorted = self.data.len() - sorted;
            if unsorted > sorted {
                // after this, it will be fully sorted. So even in the worst case 
                // it will be another self.data.len() elements until we call this again
                self.sort_and_dedup();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;
    use std::collections::*;
    use std::fmt::Debug;

    /// just a helper to get good output when a check fails
    fn unary_op<E: Debug, R: Eq + Debug>(x: E, expected: R, actual: R) -> bool {
        let res = expected == actual;
        if !res {
            println!("x:{:?} expected:{:?} actual:{:?}", x, expected, actual);
        }
        res
    }

    #[quickcheck]
    fn sort_and_dedup_check(x: Vec<i32>) -> bool {
        let expected: Vec<i32> = x
            .iter()
            .cloned()
            .collect::<BTreeSet<i32>>()
            .into_iter()
            .collect();
        let actual = sort_and_dedup(x.into_iter());
        expected == actual
    }

    #[quickcheck]
    fn dsort_and_dedup_by_check(x: Vec<(i32, i32)>) -> bool {
        // TODO: make the keep_last work!
        let expected: Vec<(i32, i32)> = x
            .iter()
            .cloned()
            .collect::<std::collections::BTreeMap<i32, i32>>()
            .into_iter()
            .collect();
        let actual = sort_and_dedup_by_key(x.iter().cloned(), |x| &x.0, Keep::Last);
        unary_op(x, expected, actual)
    }

    #[test]
    fn sort_and_dedup_by_test() {
        let v: Vec<(i32, i32)> = vec![(0, 1), (0, 2), (0, 3), (1, 1), (1, 2)];
        let keep_first = sort_and_dedup_by_key(v.clone().into_iter(), |x| &x.0, Keep::First);
        let keep_last = sort_and_dedup_by_key(v.clone().into_iter(), |x| &x.0, Keep::Last);
        assert_eq!(keep_first, vec![(0, 1), (1, 1)]);
        assert_eq!(keep_last, vec![(0, 3), (1, 2)]);
        let expected: Vec<(i32, i32)> = v
            .iter()
            .cloned()
            .collect::<std::collections::BTreeMap<i32, i32>>()
            .into_iter()
            .collect();
        println!("{:?} {:?} {:?}", keep_first, keep_last, expected)
    }
}
