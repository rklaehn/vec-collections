use std::cmp::Ord;
use std::default::Default;

struct FlipBuffer<T> {
    data: Vec<T>,
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

    pub fn copy_from_src(&mut self, n: usize) {
        if self.sb != 0 {
            let s0 = self.sb + self.sc;
            let s1 = self.data.len();
            self.data.as_mut_slice().copy_within(s0..s1, self.ti);
        }
        self.ti += n;
        self.sc += n;
    }

    pub fn copy_from(&mut self, src: &[T]) {
        let l = src.len();
        self.ensure_capacity(l);
        self.data[self.ti..self.ti + l].copy_from_slice(src);
        self.ti += l;
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

struct BinaryMerge<'a, T> {
    a: FlipBuffer<T>,
    b: &'a [T],
}

impl<'a, T: Ord + Copy + Default> BinaryMerge<'a, T> {
    fn from_a(&mut self, a0: usize, a1: usize) {
        self.a.copy_from_src(a1 - a0);
    }
    fn from_b(&mut self, b0: usize, b1: usize) {
        self.a.copy_from(&self.b[b0..b1]);
    }
    fn collision(&mut self, _ai: usize, _bi: usize) {
        self.a.copy_from_src(1);
    }
    pub fn merge(&mut self) {
        self.merge0(0, self.a.data.len(), 0, self.b.len());
    }
    fn merge0(&mut self, a0: usize, a1: usize, b0: usize, b1: usize) {
        if a0 == a1 {
            self.from_b(b0, b1)
        } else if b0 == b1 {
            self.from_a(a0, a1)
        } else {
            let am: usize = (a0 + a1) / 2;
            match self.b[b0..b1].binary_search(&self.a.src_at(am)) {
                Result::Ok(i) => {
                    let bm = i + b0;
                    // same elements. bm is the index corresponding to am
                    // merge everything below a(am) with everything below the found element
                    self.merge0(a0, am, b0, bm);
                    // add the elements a(am) and b(bm)
                    self.collision(am, bm);
                    // merge everything above a(am) with everything above the found element
                    self.merge0(am + 1, a1, bm + 1, b1);
                }
                Result::Err(i) => {
                    let bi = i + b0;
                    // not found. bi is the insertion point
                    // merge everything below a(am) with everything below the found insertion point
                    self.merge0(a0, am, b0, bi);
                    // add a(am)
                    self.from_a(am, am + 1);
                    // everything above a(am) with everything above the found insertion point
                    self.merge0(am + 1, a1, bi, b1);
                }
            }
        }
    }
}

pub fn merge<T: Ord + Default + Copy>(a: Vec<T>, b: &[T]) -> Vec<T> {
    let mut x = BinaryMerge {
        a: FlipBuffer::new(a),
        b
    };
    x.merge();
    x.a.result()
}

pub fn mergei32(a: Vec<i32>, b: &[i32]) -> Vec<i32> {
    merge(a,b)
}

#[test]
fn test_in_place_binary_merge() {
    let a = vec![2, 4, 6, 8];
    let b = vec![1, 2, 3, 5, 7];
    let r = merge(a, &b);
    println!("{:?}", r);
}
