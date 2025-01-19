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
        let new_count = self.inner().ref_count.fetch_add(1, Ordering::Relaxed);

        // TODO: handle overflow
        if new_count > usize::MAX / 2 {
            process::abort();
        }

        Self { inner: self.inner }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        let updated_count = self.inner().ref_count.fetch_sub(1, Ordering::Release);

        if updated_count == 1 {
            fence(Ordering::Acquire);

            unsafe {
                drop(Box::from_raw(self.inner.as_ptr()));
            }
        }
    }
}
