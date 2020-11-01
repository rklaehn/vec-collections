use core::cmp::Ordering;

/// Threshold above which we use the minimum comparison merge
/// For very small collections, the tape merge has a similar number of comparisons
/// and requires less state.
const MCM_THRESHOLD: usize = 8;

/// The read part of the merge state that is needed for the binary merge algorithm
/// it just needs random access for the remainder of a and b
///
/// Very often A and B are the same type, but this is not strictly necessary
pub(crate) trait MergeStateRead {
    type A;
    type B;
    /// The remaining data in a
    fn a_slice(&self) -> &[Self::A];
    /// The remaining data in b
    fn b_slice(&self) -> &[Self::B];
}

// pub(crate) trait Merger<M> {
//     fn merge(&self, m: &mut M);
// }

/// Basically a convenient to use bool to allow aborting a piece of code early using ?
/// return `None` to abort and `Some(())` to continue
pub(crate) type EarlyOut = Option<()>;

/// A binary merge operation
///
/// It is often useful to keep the merge operation and the merge state separate. E.g. computing the
/// intersection and checking if the intersection exists can be done with the same operation, but
/// a different merge state. Likewise in-place operations and operations that produce a new entity
/// can use the same merge operation. THerefore, the merge state is an additional parameter.SortedPairIter
///
/// The operation itself will often be a zero size struct
pub(crate) trait MergeOperation<M: MergeStateRead> {
    fn from_a(&self, m: &mut M, n: usize) -> EarlyOut;
    fn from_b(&self, m: &mut M, n: usize) -> EarlyOut;
    fn collision(&self, m: &mut M) -> EarlyOut;
    fn cmp(&self, a: &M::A, b: &M::B) -> Ordering;
    /// merge `an` elements from a and `bn` elements from b into the result
    ///
    /// This is a minimum comparison merge that has some overhead, so it is only worth
    /// it for larger collections and if the comparison operation is expensive.
    ///
    /// It does make a big difference e.g. when merging a very large and a very small sequence,
    /// or two disjoint sequences.
    fn merge0(&self, m: &mut M, an: usize, bn: usize) -> EarlyOut {
        if an == 0 {
            if bn > 0 {
                self.from_b(m, bn)?
            }
        } else if bn == 0 {
            if an > 0 {
                self.from_a(m, an)?
            }
        } else {
            // neither a nor b are 0
            let am: usize = an / 2;
            // pick the center element of a and find the corresponding one in b using binary search
            let a = &m.a_slice()[am];
            match m.b_slice()[..bn].binary_search_by(|b| self.cmp(a, b).reverse()) {
                Ok(bm) => {
                    // same elements. bm is the index corresponding to am
                    // merge everything below am with everything below the found element bm
                    self.merge0(m, am, bm)?;
                    // add the elements a(am) and b(bm)
                    self.collision(m)?;
                    // merge everything above a(am) with everything above the found element
                    self.merge0(m, an - am - 1, bn - bm - 1)?;
                }
                Err(bi) => {
                    // not found. bi is the insertion point
                    // merge everything below a(am) with everything below the found insertion point bi
                    self.merge0(m, am, bi)?;
                    // add a(am)
                    self.from_a(m, 1)?;
                    // everything above a(am) with everything above the found insertion point
                    self.merge0(m, an - am - 1, bn - bi)?;
                }
            }
        }
        Some(())
    }
    /// This is the classical tape merge algorithm, useful for when either
    /// the number of elements is small or the comparison operation is very cheap.
    fn tape_merge(&self, m: &mut M) -> EarlyOut {
        while !m.a_slice().is_empty() && !m.b_slice().is_empty() {
            // very convoluted way to access the first element.
            let a = &m.a_slice()[0];
            let b = &m.b_slice()[0];
            // calling the various ops advances the pointers
            match self.cmp(a, b) {
                Ordering::Equal => self.collision(m)?,
                Ordering::Less => self.from_a(m, 1)?,
                Ordering::Greater => self.from_b(m, 1)?,
            }
        }
        if !m.a_slice().is_empty() {
            self.from_a(m, m.a_slice().len())?;
        }
        if !m.b_slice().is_empty() {
            self.from_b(m, m.b_slice().len())?;
        }
        Some(())
    }
    fn merge(&self, m: &mut M) {
        let an = m.a_slice().len();
        let bn = m.b_slice().len();
        // only use the minimum comparison merge when it is worth it
        if an > MCM_THRESHOLD || bn > MCM_THRESHOLD {
            self.merge0(m, an, bn);
        } else {
            self.tape_merge(m);
        }
    }
}
