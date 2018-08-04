use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering as Memord, spin_loop_hint as cpu_relax};

use self::Memord::*;

const USIZE_MSB: usize = ::core::isize::MIN as _;

#[derive(Debug)]
pub struct Mutex(AtomicBool);

unsafe impl super::Lock for Mutex {
    const new: Self = Mutex(AtomicBool::new(false));

    #[inline]
    fn lock(&self, mut_: bool) {
        while !self.try_lock(mut_) { while self.0.load(Relaxed) { cpu_relax() } }
    }

    #[inline]
    fn unlock(&self, _: bool) { self.0.store(false, Release) }

    #[inline]
    fn try_lock(&self, _: bool) -> bool { !self.0.swap(true, Acquire) }
}

#[derive(Debug)]
pub struct RwLock(AtomicUsize);

impl RwLock {
    #[inline]
    fn read(&self) {
        while let Err(_) = {
            let mut old;
            while {
                old = self.0.load(Relaxed);
                0 != old & USIZE_MSB
            } { cpu_relax() }
            let new = old + 1;
            debug_assert_ne!(!USIZE_MSB, new);
            self.0.compare_exchange_weak(old, new, SeqCst, Relaxed)
        } { cpu_relax() }
    }

    #[inline]
    fn write(&self) {
        while let Err(_) = {
            let old = self.0.load(Relaxed) & !USIZE_MSB;
            let new = old | USIZE_MSB;
            self.0.compare_exchange_weak(old, new, SeqCst, Relaxed)
        } { cpu_relax() }
        while USIZE_MSB != self.0.load(Relaxed) { cpu_relax() }
    }
}

unsafe impl super::Lock for RwLock {
    const new: Self = RwLock(AtomicUsize::new(0));

    fn lock(&self, mut_: bool) {
        if mut_ { self.write() } else { self.read() }
    }

    #[inline]
    fn unlock(&self, mut_: bool) {
        if mut_ {
            let n = self.0.swap(0, Release);
            debug_assert_eq!(USIZE_MSB, n);
        } else {
            let n = self.0.fetch_sub(1, SeqCst);
            debug_assert_ne!(0, n & !USIZE_MSB);
        }
    }

    #[inline]
    fn try_lock(&self, mut_: bool) -> bool {
        if mut_ {
            self.0.compare_exchange(0, USIZE_MSB, SeqCst, Relaxed)
        } else {
            let old = self.0.load(Relaxed) & !USIZE_MSB;
            let new = old + 1;
            debug_assert_ne!(!USIZE_MSB, new);
            self.0.compare_exchange(old, new, SeqCst, Relaxed)
        }.is_ok()
    }
}
