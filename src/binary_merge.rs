use std::cmp::Ord;

pub(crate) trait MergeState<T> {
    fn a_slice(&self) -> &[T];
    fn b_slice(&self) -> &[T];
    fn r_slice(&self) -> &[T];
    fn move_a(&mut self, n: usize);
    fn skip_a(&mut self, n: usize);
    fn move_b(&mut self, n: usize);
    fn skip_b(&mut self, n: usize);
}

pub(crate) trait MergeOperation<'a, T: Ord, M: MergeState<T>> {
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
    fn merge(&self, m: &mut M) {
        let a1 = m.a_slice().len();
        let b1 = m.b_slice().len();
        self.merge0(m, a1, b1);
    }
}