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

    pub fn drop_from_src(&mut self, n: usize) {
        self.sc += n;
    }

    pub fn copy_from_src(&mut self, n: usize) {
        let s0 = self.sb + self.sc;
        if s0 != self.ti {
            let s1 = self.data.len();
            self.data.as_mut_slice().copy_within(s0..s1, self.ti);
        }
        self.ti += n;
        self.sc += n;
    }

    pub fn copy_from(&mut self, src: &[T], n: usize) {
        self.ensure_capacity(n);
        let l = src.len();
        self.data[self.ti..self.ti + src.len()].copy_from_slice(src);
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

trait Op<'a, T: Ord + Default + Copy> {
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

struct SetUnionOp();

impl<'a, T: Ord + Copy + Default> Op<'a, T> for SetUnionOp {
    fn from_a(&self, m: &mut  BinaryMerge<'a, T>, a0: usize, a1: usize) {
        m.a.copy_from_src(a1 - a0);
    }
    fn from_b(&self, m: &mut  BinaryMerge<'a, T>, b0: usize, b1: usize) {
        m.a.copy_from(&m.b[b0..b1], m.b.len() - b0);
    }
    fn collision(&self, m: &mut  BinaryMerge<'a, T>, _ai: usize, _bi: usize) {
        m.a.copy_from_src(1);
    }
}

struct SetIntersectionOp();

impl<'a, T: Ord + Copy + Default> Op<'a, T> for SetIntersectionOp {
    fn from_a(&self, m: &mut  BinaryMerge<'a, T>, a0: usize, a1: usize) {
        m.a.drop_from_src(a1 - a0);
    }
    fn from_b(&self, _m: &mut  BinaryMerge<'a, T>, b0: usize, b1: usize) {
    }
    fn collision(&self, m: &mut  BinaryMerge<'a, T>, ai: usize, bi: usize) {
        m.a.copy_from_src(1);
    }
}

struct SetXorOp();

impl<'a, T: Ord + Copy + Default> Op<'a, T> for SetXorOp {
    fn from_a(&self, m: &mut  BinaryMerge<'a, T>, a0: usize, a1: usize) {
        m.a.copy_from_src(a1 - a0);
    }
    fn from_b(&self, m: &mut  BinaryMerge<'a, T>, b0: usize, b1: usize) {
        m.a.copy_from(&m.b[b0..b1], m.b.len() - b0);
    }
    fn collision(&self, m: &mut  BinaryMerge<'a, T>, ai: usize, bi: usize) {
        m.a.drop_from_src(1);
    }
}

struct SetExceptOp();

impl<'a, T: Ord + Copy + Default> Op<'a, T> for SetExceptOp {
    fn from_a(&self, m: &mut  BinaryMerge<'a, T>, a0: usize, a1: usize) {
        println!("from_a {} {}", a0, a1);
        m.a.copy_from_src(a1 - a0);
    }
    fn from_b(&self, m: &mut  BinaryMerge<'a, T>, b0: usize, b1: usize) {
        println!("from_b {} {}", b0, b1);
    }
    fn collision(&self, m: &mut  BinaryMerge<'a, T>, ai: usize, bi: usize) {
        println!("collision {} {}", ai, bi);
        m.a.drop_from_src(1);
    }
}

impl<'a, T: Ord + Copy + Default> BinaryMerge<'a, T> {
    fn from_a(&mut self, a0: usize, a1: usize) {
        self.a.copy_from_src(a1 - a0);
    }
    fn from_b(&mut self, b0: usize, b1: usize) {
        self.a.copy_from(&self.b[b0..b1], self.b.len() - b0);
    }
    fn collision(&mut self, _ai: usize, _bi: usize) {
        self.a.copy_from_src(1);
    }
    pub fn merge(&mut self)  {
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
    let mut m = BinaryMerge {
        a: FlipBuffer::new(a),
        b
    };
    m.merge();
    m.a.result()
}

struct ArraySet<T>(Vec<T>);

impl<T> ArraySet<T> {
    fn single(value: T) -> Self {
        Self(vec![value])
    }
    fn into_vec(self) -> Vec<T> {
        self.0
    }
    fn as_slice(&self) -> &[T] {
        &self.0
    }
}
impl<T: Ord + Default + Copy> From<Vec<T>> for ArraySet<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}
impl<T: Ord + Default + Copy> ArraySet<T> {
    fn from_vec(vec: Vec<T>) -> Self {
        let mut vec = vec;
        vec.sort();
        Self(vec)
    }

    fn union_with(&mut self, that: &ArraySet<T>) {
        SetUnionOp().merge(&mut self.0, &that.0)
    }

    fn intersection_with(&mut self, that: &ArraySet<T>) {
        SetIntersectionOp().merge(&mut self.0, &that.0)
    }

    fn xor_with(&mut self, that: &ArraySet<T>) {
        SetXorOp().merge(&mut self.0, &that.0)
    }

    fn except(&mut self, that: &ArraySet<T>) {
        SetExceptOp().merge(&mut self.0, &that.0)
    }
}

pub fn mergei32(a: Vec<i64>, b: &[i64]) -> Vec<i64> {
    merge(a,b)
}

#[test]
fn test_in_place_binary_merge() {
    let mut a: ArraySet<usize> = vec![2, 4, 6, 8].into();
    let b: ArraySet<usize> = vec![1, 2, 3, 5, 7].into();
    let c: ArraySet<usize> = vec![1, 3].into();
    let d: ArraySet<usize> = vec![3, 4].into();
    a.union_with(&b);
    println!("{:?}", a.as_slice());
    a.intersection_with(&c);
    println!("{:?}", a.as_slice());
    a.xor_with(&d);
    println!("{:?}", a.as_slice());
    a.except(&d);
    println!("{:?}", a.as_slice());
}
