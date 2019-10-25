use std::cmp::Ord;
use std::default::Default;

pub(crate) struct FlipBuffer<T> {
    pub data: Vec<T>,
    ti: usize,
    sb: usize,
    sc: usize,
}

impl<T: Copy + Default> FlipBuffer<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self {
            data,
            ti: 0,
            sb: 0,
            sc: 0,
        }
    }

    pub fn result(self) -> Vec<T> {
        let mut vec = self.data;
        vec.truncate(self.ti);
        vec
    }

    pub fn drop_from_src(&mut self, n: usize) {
        self.sc += n;
    }

    pub fn copy_from_src(&mut self, n: usize) {
        if n > 0 {
            let s0 = self.sb + self.sc;
            if s0 != self.ti {
                let s1 = s0 + n;
                self.data.as_mut_slice().copy_within(s0..s1, self.ti);
            }
            self.ti += n;
            self.sc += n;
        }
    }

    pub fn copy_from(&mut self, src: &[T], n: usize) {
        if n > 0 {
            self.ensure_capacity(n);
            let l = src.len();
            self.data[self.ti..self.ti + src.len()].copy_from_slice(src);
            self.ti += l;
        }
    }

    pub fn src_at(&self, o: usize) -> &T {
        debug_assert!(o >= self.sc);
        &self.data[self.sb + o]
    }

    fn ensure_capacity(&mut self, required: usize) {
        let si = self.sb + self.sc;
        let capacity = si - self.ti;
        if capacity < required {
            let missing = required - capacity;
            self.data
                .splice(si..si, std::iter::repeat(T::default()).take(missing));
            self.sb += missing;
        }
    }
}

pub(crate) struct BinaryMerge<'a, T> {
    pub a: FlipBuffer<T>,
    pub b: &'a [T],
}

pub(crate) trait Op<'a, T: Ord + Default + Copy> {
    fn from_a(&self, m: &mut BinaryMerge<'a, T>, a: usize);
    fn from_b(&self, m: &mut BinaryMerge<'a, T>, b0: usize, b1: usize);
    fn collision(&self, m: &mut BinaryMerge<'a, T>);
    fn merge0(&self, m: &mut BinaryMerge<'a, T>, a0: usize, a1: usize, b0: usize, b1: usize) {
        if a0 == a1 {
            self.from_b(m, b0, b1)
        } else if b0 == b1 {
            self.from_a(m, a1 - a0)
        } else {
            let am: usize = (a0 + a1) / 2;
            match m.b[b0..b1].binary_search(&m.a.src_at(am)) {
                Result::Ok(i) => {
                    let bm = i + b0;
                    // same elements. bm is the index corresponding to am
                    // merge everything below a(am) with everything below the found element
                    self.merge0(m, a0, am, b0, bm);
                    // add the elements a(am) and b(bm)
                    self.collision(m);
                    // merge everything above a(am) with everything above the found element
                    self.merge0(m, am + 1, a1, bm + 1, b1);
                }
                Result::Err(i) => {
                    let bi = i + b0;
                    // not found. bi is the insertion point
                    // merge everything below a(am) with everything below the found insertion point
                    self.merge0(m, a0, am, b0, bi);
                    // add a(am)
                    self.from_a(m, 1);
                    // everything above a(am) with everything above the found insertion point
                    self.merge0(m, am + 1, a1, bi, b1);
                }
            }
        }
    }
    fn merge(&self, a: &mut Vec<T>, b: &'a [T]) {
        let al = a.len();
        let mut t: Vec<T> = Vec::new();
        std::mem::swap(&mut t, a);
        let mut state: BinaryMerge<'a, T> = BinaryMerge {
            a: FlipBuffer::new(t),
            b,
        };
        self.merge0(&mut state, 0, al, 0, b.len());
        *a = state.a.result();
    }
}

pub(crate) trait Op2<'a, T: Ord + Default + Copy, M: MergeState<T>> {
    fn from_a(&self, m: &mut M, n: usize);
    fn from_b(&self, m: &mut M, n: usize);
    fn collision(&self, m: &mut M);
    fn merge0(&self, m: &mut M, a: usize, b: usize) {
        if a == 0 {
            self.from_b(m, b)
        } else if b == 0 {
            self.from_a(m, a)
        } else {
            // neither a nor b are 0
            let am: usize = a / 2;
            // pick the center element of a and find the corresponding one in b using binary search
            match m.b_slice()[0..b].binary_search(&m.a_slice()[am]) {
                Result::Ok(bm) => {
                    // same elements. bm is the index corresponding to am
                    // merge everything below am with everything below the found element bm
                    self.merge0(m, am, bm);
                    // add the elements a(am) and b(bm)
                    self.collision(m);
                    // merge everything above a(am) with everything above the found element
                    self.merge0(m, a - am - 1, b - bm - 1);
                }
                Result::Err(bi) => {
                    // not found. bi is the insertion point
                    // merge everything below a(am) with everything below the found insertion point bi
                    self.merge0(m, am, bi);
                    // add a(am)
                    self.from_a(m, 1);
                    // everything above a(am) with everything above the found insertion point
                    self.merge0(m, a - am - 1, b - bi);
                }
            }
        }
    }
    fn merge2(&self, m: &mut M) {
        let a1 = m.a_slice().len();
        let b1 = m.b_slice().len();
        self.merge0(m, a1, b1);
    }
}

pub(crate) trait MergeState<T> {
    fn a_slice(&self) -> &[T];
    fn b_slice(&self) -> &[T];
    fn r_slice(&self) -> &[T];
    fn move_a(&mut self, n: usize);
    fn skip_a(&mut self, n: usize);
    fn move_b(&mut self, n: usize);
    fn skip_b(&mut self, n: usize);
}

pub(crate) struct InPlaceMergeState<'a, T> {
    a: Vec<T>,
    b: &'a [T],
    // number of result elements
    rn: usize,
    // base of the remaining stuff in a
    ab: usize,
}

impl<'a, T: Copy + Default + Ord> InPlaceMergeState<'a, T> {
    pub fn new(a: Vec<T>, b: &'a [T]) -> Self {
        Self { a, b, rn: 0, ab: 0 }
    }

    pub fn merge<O: Op2<'a, T, Self>>(a: &mut Vec<T>, b: &'a [T], o: O) {
        let mut t: Vec<T> = Default::default();
        std::mem::swap(a, &mut t);
        let mut state = InPlaceMergeState::new(t, b);
        o.merge2(&mut state);
        *a = state.a;
    }

    fn ensure_capacity(&mut self, required: usize) {
        let rn = self.rn;
        let ab = self.ab;
        let capacity = ab - rn;
        if capacity < required {
            let missing = required - capacity;
            let fill = T::default();
            self.a.splice(ab..ab, std::iter::repeat(fill).take(missing));
            self.ab += missing;
        }
    }
}

impl<'a, T: Copy + Default + Ord> MergeState<T> for InPlaceMergeState<'a, T> {
    fn a_slice(&self) -> &[T] {
        let a0 = self.ab;
        let a1 = self.a.len();
        &self.a[a0..a1]
    }
    fn b_slice(&self) -> &[T] {
        self.b
    }
    fn r_slice(&self) -> &[T] {
        &self.a[0..self.rn]
    }
    fn move_a(&mut self, n: usize) {
        if n > 0 {
            if self.ab != self.rn {
                let a0 = self.ab;
                let a1 = a0 + n;
                self.a.as_mut_slice().copy_within(a0..a1, self.rn);
            }
            self.ab += n;
            self.rn += n;
        }
    }
    fn skip_a(&mut self, n: usize) {
        self.ab += n;
    }
    fn move_b(&mut self, n: usize) {
        if n > 0 {
            self.ensure_capacity(n);
            let r0 = self.rn;
            let r1 = self.rn + n;
            self.a[r0..r1].copy_from_slice(&self.b[0..n]);
            self.rn += n;
        }
    }
    fn skip_b(&mut self, n: usize) {
        let b0 = n;
        let b1 = self.b.len();
        self.b = &self.b[b0..b1];
    }
}
