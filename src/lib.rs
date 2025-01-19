pub mod arc;

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        thread,
    };

    use crate::arc::Arc;

    #[test]
    fn it_works() {
        static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter {
            value: &'static str,
        }

        impl Drop for DropCounter {
            fn drop(&mut self) {
                NUM_DROPS.fetch_add(1, Ordering::Relaxed);
            }
        }

        let arc = Arc::new(DropCounter { value: "hello" });
        let cloned_arc = arc.clone();

        let t = thread::spawn(move || {
            assert_eq!(arc.value, "hello");
        });

        assert_eq!(cloned_arc.value, "hello");

        t.join().unwrap();

        assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 0);

        drop(cloned_arc);

        assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 1);
    }
}
