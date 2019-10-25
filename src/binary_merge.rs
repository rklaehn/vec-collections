use std::cmp::Ord;
use std::default::Default;
use std::fmt::Debug;

pub(crate) trait MergeOperation<'a, T: Ord + Default + Copy, M: MergeState<T>> {
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

impl<'a, T: Copy + Default + Debug> Debug for InPlaceMergeState<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a: {:?}, b: {:?}, r: {:?}", self.a_slice(), self.b_slice(), self.r_slice())
    }
}

impl<'a, T: Copy + Default + Ord> InPlaceMergeState<'a, T> {

    pub fn merge<O: MergeOperation<'a, T, Self>>(a: &mut Vec<T>, b: &'a [T], o: O) {
        let mut t: Vec<T> = Default::default();
        std::mem::swap(a, &mut t);
        let mut state = InPlaceMergeState::new(t, b);
        o.merge2(&mut state);
        *a = state.into_vec();
    }
}

impl<'a, T: Copy + Default> InPlaceMergeState<'a, T> {
    pub fn new(a: Vec<T>, b: &'a [T]) -> Self {
        Self { a, b, rn: 0, ab: 0 }
    }

    pub fn into_vec(self) -> Vec<T> {
        let mut r = self.a; 
        r.truncate(self.rn);
        r
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

impl<'a, T: Copy + Default> MergeState<T> for InPlaceMergeState<'a, T> {
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
            self.skip_b(n);
            self.rn += n;
        }
    }
    fn skip_b(&mut self, n: usize) {
        let b0 = n;
        let b1 = self.b.len();
        self.b = &self.b[b0..b1];
    }
}
