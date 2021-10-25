use core::cell::UnsafeCell;
use parking_lot::Mutex;

/// Utility for a lazily initialized value
#[derive(Default)]
pub(crate) struct Lazy<A, B> {
    mutex: Mutex<()>,
    data: UnsafeCell<Either<A, B>>,
}

impl<A: Clone, B: Clone> Clone for Lazy<A, B> {
    fn clone(&self) -> Self {
        let guard = self.mutex.lock();
        let data = unsafe { (&*self.data.get()).clone() };
        drop(guard);
        Self {
            mutex: Mutex::new(()),
            data: UnsafeCell::new(data),
        }
    }
}

#[derive(Debug, Clone)]
enum Either<A, B> {
    A(A),
    B(B),
}

impl<A: Default, B> Default for Either<A, B> {
    fn default() -> Self {
        Self::A(A::default())
    }
}

impl<A: Copy, B> Either<A, B> {
    fn a_to_b(&mut self, f: impl Fn(A) -> B) {
        if let Either::A(a) = self {
            *self = Either::B(f(*a))
        }
    }
}

impl<A: Copy, B> Lazy<A, B> {
    pub fn uninitialized(data: A) -> Self {
        Self::new(Either::A(data))
    }

    pub fn initialized(data: B) -> Self {
        Self::new(Either::B(data))
    }

    pub fn get_or_create(&self, f: impl Fn(A) -> B) -> &B {
        unsafe {
            let guard = self.mutex.lock();
            let data: &mut Either<A, B> = &mut *self.data.get();
            data.a_to_b(f);
            drop(guard);
            if let Either::B(data) = &*self.data.get() {
                data
            } else {
                panic!()
            }
        }
    }

    pub fn get_or_create_mut(&mut self, f: impl Fn(A) -> B) -> &mut B {
        unsafe {
            let guard = self.mutex.lock();
            let data: &mut Either<A, B> = &mut *self.data.get();
            data.a_to_b(f);
            drop(guard);
            if let Either::B(data) = &mut *self.data.get() {
                data
            } else {
                panic!()
            }
        }
    }

    fn new(data: Either<A, B>) -> Self {
        Self {
            mutex: Mutex::new(()),
            data: UnsafeCell::new(data),
        }
    }
}
