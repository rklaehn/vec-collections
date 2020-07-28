use std::cmp::{min, Ordering};

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

fn dedup<T: Eq>(d: &mut [T], keep: Keep) -> usize {
    dedup_by(d, T::eq, keep)
}

/// size of a chunk for dedup and sort. After we have a full chunk we will sort it in.
const CHUNK_BITS: u32 = 3;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Keep {
    First,
    Last,
}

trait Inner<T>: AsMut<T> {
    fn push(&mut self, value: T);
}

/// an aggregator to incrementally sort and deduplicate unsorted elements
///
/// this is a compromise between sorting and deduping at the end, which can have a lot of
/// temporary memory usage if you feed it with lots of duplicate elements, and sorting
/// on each insertion, which is expensive for a flat data structure due to all the memory
/// movements.
struct SortAndDedup<T, F> {
    /// partially sorted and deduplicated data elements
    data: Vec<T>,
    /// number of sorted elements
    sorted: usize,
    /// comparison
    cmp: F,
    /// total number of unsorted elements that have been added
    count: usize,
    /// which element to keep in case of duplicates
    keep: Keep,
}

pub fn sort_and_dedup<T: Ord, I: Iterator<Item = T>>(iter: I) -> Vec<T> {
    let mut agg: SortAndDedup<T, _> = SortAndDedup {
        data: Vec::with_capacity(iter.size_hint().0),
        count: 0,
        sorted: 0,
        cmp: |a: &T, b: &T| a.cmp(b),
        keep: Keep::First,
    };
    for x in iter {
        agg.push(x);
    }
    agg.into_vec()
}

pub fn sort_and_dedup_by_key<T, K, I, F>(iter: I, key: F, keep: Keep) -> Vec<T>
where
    K: Ord,
    I: Iterator<Item = T>,
    F: Fn(&T) -> &K,
{
    let mut agg: SortAndDedup<T, _> = SortAndDedup {
        data: Vec::with_capacity(min(iter.size_hint().0, 16)),
        count: 0,
        sorted: 0,
        cmp: |a: &T, b: &T| key(a).cmp(&key(b)),
        keep,
    };
    for x in iter {
        agg.push(x);
    }
    agg.into_vec()
}

impl<T, F> SortAndDedup<T, F>
where
    F: Fn(&T, &T) -> Ordering,
{
    fn sort_and_dedup(&mut self) {
        if self.sorted < self.data.len() {
            let cmp = &self.cmp;
            let slice = self.data.as_mut_slice();
            slice.sort_by(cmp);
            let unique = dedup_by(slice, |a, b| cmp(a, b) == Ordering::Equal, self.keep);
            self.data.truncate(unique);
            self.sorted = self.data.len();
        }
    }

    fn into_vec(self) -> Vec<T> {
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
                self.data.push(elem);
                self.sorted += 1;
            }
        } else {
            // not sorted
            self.data.push(elem);
        }
        let level = self.count.trailing_zeros();
        if level >= CHUNK_BITS {
            let sorted = self.sorted;
            let unsorted = self.data.len() - sorted;
            if unsorted > sorted {
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
