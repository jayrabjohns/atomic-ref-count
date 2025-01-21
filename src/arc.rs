use std::{
    ops::Deref,
    process,
    ptr::NonNull,
    sync::atomic::{fence, AtomicUsize, Ordering},
};

struct InnerArc<T> {
    ref_count: AtomicUsize,
    value: T,
}

pub struct Arc<T> {
    inner: NonNull<InnerArc<T>>,
}

unsafe impl<T: Send + Sync> Send for Arc<T> {}

unsafe impl<T: Send + Sync> Sync for Arc<T> {}

impl<T> Arc<T> {
    pub fn new(value: T) -> Arc<T> {
        let inner = InnerArc {
            ref_count: 1.into(),
            value,
        };

        Arc {
            inner: NonNull::from(Box::leak(Box::new(inner))),
        }
    }

    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        let count: usize = arc.inner().ref_count.load(Ordering::Relaxed);

        if count == 1 {
            fence(Ordering::Acquire);

            unsafe { Some(&mut arc.inner.as_mut().value) }
        } else {
            None
        }
    }

    fn inner(&self) -> &InnerArc<T> {
        unsafe { self.inner.as_ref() }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner().value
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        let count: usize = self.inner().ref_count.fetch_add(1, Ordering::Relaxed);

        // TODO: handle overflow
        if count > usize::MAX / 2 {
            process::abort();
        }

        Self { inner: self.inner }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        let count: usize = self.inner().ref_count.fetch_sub(1, Ordering::Release);

        if count == 1 {
            fence(Ordering::Acquire);

            unsafe {
                drop(Box::from_raw(self.inner.as_ptr()));
            }
        }
    }
}
