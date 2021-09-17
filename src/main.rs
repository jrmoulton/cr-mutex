#![allow(dead_code)]
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::spawn;

const LOCKED: bool = true;
const UNLOCKED: bool = false;

pub struct Mujex<T> {
    locked: AtomicBool,
    v: UnsafeCell<T>,
}

unsafe impl<T> Sync for Mujex<T> where T: Send {}

impl<T> Mujex<T> {
    pub fn new(t: T) -> Self {
        Self {
            locked: AtomicBool::new(UNLOCKED),
            v: UnsafeCell::new(t),
        }
    }

    fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self
            .locked
            .compare_exchange(UNLOCKED, LOCKED, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {}
        self.locked.store(LOCKED, Ordering::SeqCst);
        // f is returning the value of an unsafeCell.get which is an unsafe raw mutable pointer
        let ret = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::SeqCst);
        // we then pass that raw pointer back out
        ret
    }
}

fn main() {
    let l = Arc::new(Mujex::new(0));
    let handles: Vec<_> = (0..100)
        .map(|_| {
            let l_clone = l.clone();
            let _ = 5;
            spawn(move || {
                for _ in 0..1000 {
                    l_clone.with_lock(|v| {
                        *v += 1;
                    });
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
    assert_eq!(l.with_lock(|v| *v), 100 * 1000);
}
