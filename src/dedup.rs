/// deduplicate a slice, moving the duplicates to the end.
/// returns the number of unique elements.
///
/// there is an unstable library feature called slice.partition_dedup which is
/// identical: https://github.com/rust-lang/rust/issues/54279
pub fn dedup<T: Eq>(d: &mut [T]) -> usize {
    if !d.is_empty() {
        let mut j = 0;
        for i in 1..d.len() {
            if d[i] != d[j] {
                j += 1;
            }
            if i != j {
                d.swap(i, j);
            }
        }
        j + 1
    } else {
        0
    }
}

/// an aggregator to incrementally sort and deduplicate unsorted elements
pub(crate) struct SortAndDedup<T> {
    /// sorted slices
    slices: Vec<(usize, u32)>,
    /// partially sorted and deduplicated data elements
    data: Vec<T>,
    /// total number of unsorted elements that have been added
    count: usize,
}

/// size of a chunk for dedup and sort. After we have a full chunk we will sort it in.
const CHUNK_BITS: u32 = 3;

impl<T: Ord> SortAndDedup<T> {
    pub fn new() -> Self {
        Self {
            slices: Vec::new(),
            data: Vec::new(),
            count: 0,
        }
    }

    fn add(&mut self, level: u32) -> usize {
        let mut total = 0;
        while let Some((s, l)) = self.slices.last() {
            if *l > level {
                break;
            } else {
                total += s;
                self.slices.pop();
            }
        }
        total
    }

    pub fn result(self) -> Vec<T> {
        let mut res = self.data;
        res.sort();
        res.dedup();
        res
    }

    pub fn push(&mut self, elem: T) {
        self.data.push(elem);
        self.count += 1;
        let level = self.count.trailing_zeros();
        if level >= CHUNK_BITS {
            let to_sort = self.add(level) + (1 << CHUNK_BITS);
            let i1 = self.data.len();
            let i0 = i1 - to_sort;
            let slice = &mut self.data[i0..i1];
            slice.sort();
            let remaining = dedup(slice);
            self.data.truncate(i0 + remaining);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[quickcheck]
    fn sort_and_dedup_check(elems: Vec<i32>) -> bool {
        let mut expected = elems.clone();
        expected.sort();
        expected.dedup();
        let mut agg = SortAndDedup::<i32>::new();
        for elem in elems {
            agg.push(elem);
        }
        let actual = agg.result();
        expected == actual
    }

    #[quickcheck]
    fn dedup_check(x: Vec<i32>) -> bool {
        let mut y = x;
        y.sort();
        let mut expected = y.clone();
        expected.dedup();
        let expected = expected.as_slice();
        let actual = y.as_mut_slice();
        let n = dedup(actual);
        let actual = &actual[0..n];
        expected == actual
    }
}
