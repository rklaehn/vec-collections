use crate::binary_merge::{EarlyOut, MergeOperation, MergeStateRead};
use crate::iterators::SliceIterator;
use crate::small_vec_builder::InPlaceSmallVecBuilder;
use core::{fmt, fmt::Debug};
use smallvec::{Array, SmallVec};
use std::marker::PhantomData;

/// A typical write part for the merge state
pub(crate) trait MergeStateMut: MergeStateRead {
    /// Consume n elements of a
    fn advance_a(&mut self, n: usize, take: bool) -> EarlyOut;
    /// Consume n elements of b
    fn advance_b(&mut self, n: usize, take: bool) -> EarlyOut;
}

pub(crate) trait MutateInput: MergeStateMut {
    fn source_slices_mut(&mut self) -> (&mut [Self::A], &[Self::B]);
}

pub(crate) struct InPlaceMergeState<
    'a,
    A: Array,
    B: Array,
    C: Converter<B::Item, A::Item> = NoConverter,
> {
    pub a: InPlaceSmallVecBuilder<'a, A>,
    pub b: smallvec::IntoIter<B>,
    _c: PhantomData<C>,
}

impl<'a, A: Array, B: Array, C: Converter<B::Item, A::Item>> InPlaceMergeState<'a, A, B, C> {
    fn new(a: &'a mut SmallVec<A>, b: SmallVec<B>) -> Self {
        Self {
            a: a.into(),
            b: b.into_iter(),
            _c: PhantomData,
        }
    }
}

impl<'a, A: Array, B: Array, C: Converter<B::Item, A::Item>> MergeStateRead
    for InPlaceMergeState<'a, A, B, C>
{
    type A = A::Item;
    type B = B::Item;
    fn a_slice(&self) -> &[A::Item] {
        self.a.source_slice()
    }
    fn b_slice(&self) -> &[B::Item] {
        self.b.as_slice()
    }
}

impl<'a, A: Array, B: Array, C: Converter<B::Item, A::Item>> MergeStateMut
    for InPlaceMergeState<'a, A, B, C>
{
    fn advance_a(&mut self, n: usize, take: bool) -> EarlyOut {
        self.a.consume(n, take);
        Some(())
    }
    fn advance_b(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.a.extend_from_iter::<_, C>(&mut self.b, n);
        } else {
            for _ in 0..n {
                let _ = self.b.next();
            }
        }
        Some(())
    }
}

impl<'a, A: Array, B: Array, C: Converter<B::Item, A::Item>> InPlaceMergeState<'a, A, B, C> {
    pub fn merge<O: MergeOperation<Self>>(a: &'a mut SmallVec<A>, b: SmallVec<B>, o: O, _c: C) {
        let mut state = Self::new(a, b);
        o.merge(&mut state);
    }
}

/// An in place merge state where the rhs is a reference
pub(crate) struct InPlaceMergeStateRef<'a, A: Array, B, C: Converter<&'a B, A::Item> = NoConverter>
{
    pub(crate) a: InPlaceSmallVecBuilder<'a, A>,
    pub(crate) b: SliceIterator<'a, B>,
    _c: PhantomData<C>,
}

impl<'a, A: Array, B, C: Converter<&'a B, A::Item>> InPlaceMergeStateRef<'a, A, B, C> {
    pub fn new(a: &'a mut SmallVec<A>, b: &'a impl AsRef<[B]>) -> Self {
        Self {
            a: a.into(),
            b: SliceIterator(b.as_ref()),
            _c: PhantomData,
        }
    }
}

impl<'a, A: Array, B, C: Converter<&'a B, A::Item>> MergeStateRead
    for InPlaceMergeStateRef<'a, A, B, C>
{
    type A = A::Item;
    type B = B;
    fn a_slice(&self) -> &[A::Item] {
        self.a.source_slice()
    }
    fn b_slice(&self) -> &[B] {
        self.b.as_slice()
    }
}

impl<'a, A: Array, B, C: Converter<&'a B, A::Item>> MergeStateMut
    for InPlaceMergeStateRef<'a, A, B, C>
where
    A::Item: Clone,
{
    fn advance_a(&mut self, n: usize, take: bool) -> EarlyOut {
        self.a.consume(n, take);
        Some(())
    }
    fn advance_b(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.a.extend_from_ref_iter::<_, C>(&mut self.b, n);
        } else {
            for _ in 0..n {
                let _ = self.b.next();
            }
        }
        Some(())
    }
}

impl<'a, A, B, C: Converter<&'a B, A::Item>> MutateInput for InPlaceMergeStateRef<'a, A, B, C>
where
    A: Array,
    A::Item: Clone,
{
    fn source_slices_mut(&mut self) -> (&mut [Self::A], &[Self::B]) {
        (self.a.source_slice_mut(), self.b.as_slice())
    }
}

impl<'a, A: Array, B: 'a, C: Converter<&'a B, A::Item>> InPlaceMergeStateRef<'a, A, B, C> {
    pub fn merge<O: MergeOperation<Self>>(
        a: &'a mut SmallVec<A>,
        b: &'a impl AsRef<[B]>,
        o: O,
        _c: C,
    ) {
        let mut state = Self::new(a, b);
        o.merge(&mut state);
    }
}

/// A merge state where we only track if elements have been produced, and abort as soon as the first element is produced
pub(crate) struct BoolOpMergeState<'a, A, B> {
    a: SliceIterator<'a, A>,
    b: SliceIterator<'a, B>,
    r: bool,
}

impl<'a, A: Debug, B: Debug> Debug for BoolOpMergeState<'a, A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "a: {:?}, b: {:?} r: {}",
            self.a_slice(),
            self.b_slice(),
            self.r
        )
    }
}

impl<'a, A, B> BoolOpMergeState<'a, A, B> {
    fn new(a: &'a [A], b: &'a [B]) -> Self {
        Self {
            a: SliceIterator(a),
            b: SliceIterator(b),
            r: false,
        }
    }
}

impl<'a, A, B> BoolOpMergeState<'a, A, B> {
    pub fn merge<O: MergeOperation<Self>>(a: &'a [A], b: &'a [B], o: O) -> bool {
        let mut state = Self::new(a, b);
        o.merge(&mut state);
        state.r
    }
}

impl<'a, A, B> MergeStateRead for BoolOpMergeState<'a, A, B> {
    type A = A;
    type B = B;
    fn a_slice(&self) -> &[A] {
        self.a.as_slice()
    }
    fn b_slice(&self) -> &[B] {
        self.b.as_slice()
    }
}

impl<'a, A, B> MergeStateMut for BoolOpMergeState<'a, A, B> {
    fn advance_a(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.r = true;
            None
        } else {
            self.a.drop_front(n);
            Some(())
        }
    }
    fn advance_b(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.r = true;
            None
        } else {
            self.b.drop_front(n);
            Some(())
        }
    }
}

pub trait Converter<A, B> {
    fn convert(value: A) -> B;
}

/// A converter that does not work. Use this only if you are sure it will never be used.
pub struct NoConverter;

impl<A, B> Converter<A, B> for NoConverter {
    fn convert(_: A) -> B {
        panic!("conversion not possible")
    }
}

/// The clone converter that clones the value
pub struct CloneConverter;

impl<A: Clone> Converter<&A, A> for CloneConverter {
    fn convert(value: &A) -> A {
        value.clone()
    }
}

/// The identity converter that just passes through the value
pub struct IdConverter;

impl<A> Converter<A, A> for IdConverter {
    fn convert(value: A) -> A {
        value
    }
}

/// A merge state where we build into a new vector
pub(crate) struct SmallVecMergeState<'a, A, B, Arr: Array, C: Converter<&'a B, A> = NoConverter> {
    pub a: SliceIterator<'a, A>,
    pub b: SliceIterator<'a, B>,
    pub r: SmallVec<Arr>,
    _c: PhantomData<C>,
}

impl<'a, A: Debug, B: Debug, Arr: Array> Debug for SmallVecMergeState<'a, A, B, Arr> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a: {:?}, b: {:?}", self.a_slice(), self.b_slice(),)
    }
}

impl<'a, A, B, Arr: Array, C: Converter<&'a B, A>> SmallVecMergeState<'a, A, B, Arr, C> {
    fn new(a: &'a [A], b: &'a [B], r: SmallVec<Arr>) -> Self {
        Self {
            a: SliceIterator(a),
            b: SliceIterator(b),
            r,
            _c: PhantomData,
        }
    }

    pub fn into_vec(self) -> SmallVec<Arr> {
        self.r
    }

    pub fn merge<O: MergeOperation<Self>>(a: &'a [A], b: &'a [B], o: O, _c: C) -> SmallVec<Arr> {
        let t: SmallVec<Arr> = SmallVec::new();
        let mut state = Self::new(a, b, t);
        o.merge(&mut state);
        state.into_vec()
    }
}

impl<'a, A, B, Arr: Array, C: Converter<&'a B, A>> MergeStateRead
    for SmallVecMergeState<'a, A, B, Arr, C>
{
    type A = A;
    type B = B;
    fn a_slice(&self) -> &[A] {
        self.a.as_slice()
    }
    fn b_slice(&self) -> &[B] {
        self.b.as_slice()
    }
}

impl<'a, A: Clone, B, Arr: Array<Item = A>, C: Converter<&'a B, A>> MergeStateMut
    for SmallVecMergeState<'a, A, B, Arr, C>
{
    fn advance_a(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.r.reserve(n);
            for e in self.a.take_front(n).iter() {
                self.r.push(e.clone())
            }
        } else {
            self.a.drop_front(n);
        }
        Some(())
    }
    fn advance_b(&mut self, n: usize, take: bool) -> EarlyOut {
        if take {
            self.r.reserve(n);
            for e in self.b.take_front(n).iter() {
                self.r.push(C::convert(e))
            }
        } else {
            self.b.drop_front(n);
        }
        Some(())
    }
}
