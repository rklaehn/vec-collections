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
    fn from_a(&self, m: &mut BinaryMerge<'a, T>, a0: usize, a1: usize);
    fn from_b(&self, m: &mut BinaryMerge<'a, T>, b0: usize, b1: usize);
    fn collision(&self, m: &mut BinaryMerge<'a, T>, ai: usize, bi: usize);
    fn merge0(&self, m: &mut BinaryMerge<'a, T>, a0: usize, a1: usize, b0: usize, b1: usize) {
        if a0 == a1 {
            self.from_b(m, b0, b1)
        } else if b0 == b1 {
            self.from_a(m, a0, a1)
        } else {
            let am: usize = (a0 + a1) / 2;
            match m.b[b0..b1].binary_search(&m.a.src_at(am)) {
                Result::Ok(i) => {
                    let bm = i + b0;
                    // same elements. bm is the index corresponding to am
                    // merge everything below a(am) with everything below the found element
                    self.merge0(m, a0, am, b0, bm);
                    // add the elements a(am) and b(bm)
                    self.collision(m, am, bm);
                    // merge everything above a(am) with everything above the found element
                    self.merge0(m, am + 1, a1, bm + 1, b1);
                }
                Result::Err(i) => {
                    let bi = i + b0;
                    // not found. bi is the insertion point
                    // merge everything below a(am) with everything below the found insertion point
                    self.merge0(m, a0, am, b0, bi);
                    // add a(am)
                    self.from_a(m, am, am + 1);
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
