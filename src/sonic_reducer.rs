pub fn reduce<T, I: IntoIterator<Item = T>, F: Fn(&mut T, T)>(iter: I, op: F) -> Option<T> {
    let mut reducer: SonicReducer<T, F> = SonicReducer {
        buffers: Vec::new(),
        count: 0,
        op,
    };
    for e in iter.into_iter() {
        reducer.add(Some(e));
    }
    reducer.result()
}

struct SonicReducer<T, F> {
    buffers: Vec<Option<T>>,
    count: usize,
    op: F,
}

fn zero_and_get<T>(a: &mut Option<T>) -> Option<T> {
    if a.is_some() {
        let mut a1: Option<T> = None;
        std::mem::swap(a, &mut a1);
        a1
    } else {
        None
    }
}

fn adsorb<T, F: Fn(&mut T, T)>(op: &F, a: &mut Option<T>, b: Option<T>) -> Option<T> {
    if let Some(mut a) = zero_and_get(a) {
        if let Some(b) = b {
            op(&mut a, b);
        }
        Some(a)
    } else {
        b
    }
}

impl<T, F: Fn(&mut T, T)> SonicReducer<T, F> {
    // ensures that we can assign at the given index
    fn reserve(&mut self, index: usize) {
        // make sure buffer is large enough
        while self.buffers.len() < index + 1 {
            self.buffers.push(None);
        }
    }
    fn reduce_to(&mut self, level: usize, value: Option<T>) -> Option<T> {
        // adsorb the value up to level
        let mut result = value;
        for i in 0..level {
            result = adsorb(&self.op, &mut self.buffers[i], result);
        }
        result
    }
    pub fn add(&mut self, value: Option<T>) {
        self.count += 1;
        let level = self.count.trailing_zeros() as usize;
        self.reserve(level);
        self.buffers[level] = self.reduce_to(level, value);
    }
    pub fn result(&mut self) -> Option<T> {
        self.reduce_to(self.buffers.len(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[quickcheck]
    fn sum(elems: Vec<i32>) -> bool {
        let expected: i32 = elems.iter().sum();
        let actual = reduce(elems, |a, b| *a += b).unwrap_or(0);
        expected == actual
    }

    #[quickcheck]
    fn concat(elems: Vec<Vec<u8>>) -> bool {
        let mut expected: Vec<u8> = Vec::new();
        for elem in elems.clone() {
            expected.extend(elem);
        }
        let actual = reduce(elems, |a, b| a.extend(&b)).unwrap_or(Vec::new());
        expected == actual
    }

    #[test]
    fn reduce_numbers() {
        assert_eq!(4950, reduce(1..100, |a, b| *a += b).unwrap());
    }
}
