#![no_std]

#![feature(const_fn)]

use core::{cell::UnsafeCell, ops::{Deref, DerefMut}};

pub mod raw;

/// Mutual exclusionary primitive
pub struct RwLock<T: ?Sized, Lock> {
    lock: Lock,
    valu: UnsafeCell<T>,
}

impl<T, Lock: raw::Lock> RwLock<T, Lock> {
    #[inline]
    pub const fn new(x: T) -> Self {
        Self {
            lock: Lock::new,
            valu: UnsafeCell::new(x),
        }
    }
}

impl<T: ?Sized, Lock: raw::Lock> RwLock<T, Lock> {
    /// Take a reference to the guarded value, blocking if another thread is already
    /// holding an exclusive reference.
    #[inline]
    pub fn lock(&self) -> Guard<T, Lock> { unsafe {
        self.lock.lock(false);
        Guard {
            lock: &self.lock,
            valu: &*self.valu.get(),
        }
    } }

    /// Take an exclusive reference to the guarded value, blocking if another thread is already
    /// holding a reference.
    #[inline]
    pub fn lock_mut(&self) -> GuardMut<T, Lock> { unsafe {
        self.lock.lock(true);
        GuardMut {
            lock: &self.lock,
            valu: &mut *self.valu.get(),
        }
    } }

    /// Take a reference to the guarded value, returning `None` if another thread is already
    /// holding an exclusive reference.
    #[inline]
    pub fn try_lock(&self) -> Option<Guard<T, Lock>> { unsafe {
        if self.lock.try_lock(false) {
            Some(Guard {
                lock: &self.lock,
                valu: &*self.valu.get(),
            })
        } else { None }
    } }

    /// Take an exclusive reference to the guarded value, returning `None` if another thread is
    /// already holding a reference.
    #[inline]
    pub fn try_lock_mut(&self) -> Option<GuardMut<T, Lock>> { unsafe {
        if self.lock.try_lock(true) {
            Some(GuardMut {
                lock: &self.lock,
                valu: &mut *self.valu.get(),
            })
        } else { None }
    } }
}

/// Reference to guarded value
pub struct Guard<'a, T: ?Sized + 'a, Lock: 'a + raw::Lock> {
    lock: &'a Lock,
    valu: &'a T,
}

impl<'a, T: ?Sized, Lock: raw::Lock> Deref for Guard<'a, T, Lock> {
    type Target = T;
    #[inline] fn deref(&self) -> &T { self.valu }
}

impl<'a, T: ?Sized, Lock: raw::Lock> Drop for Guard<'a, T, Lock> {
    #[inline] fn drop(&mut self) { self.lock.unlock(false) }
}

impl<'a, T: ?Sized, Lock: raw::Lock> Guard<'a, T, Lock> {
    #[inline]
    pub fn try_upgrade(self) -> Result<GuardMut<'a, T, Lock>, Guard<'a, T, Lock>> {
        if !self.lock.try_upgrade() { Err(self) } else { Ok(GuardMut {
            lock: self.lock,
            valu: unsafe { &mut *(self.valu as *const T as *mut T) },
        }) }
    }
}

/// Exclusive reference to guarded value
pub struct GuardMut<'a, T: ?Sized + 'a, Lock: 'a + raw::Lock> {
    lock: &'a Lock,
    valu: &'a mut T,
}

impl<'a, T: ?Sized, Lock: raw::Lock> Deref for GuardMut<'a, T, Lock> {
    type Target = T;
    #[inline] fn deref(&self) -> &T { self.valu }
}

impl<'a, T: ?Sized, Lock: raw::Lock> DerefMut for GuardMut<'a, T, Lock> {
    #[inline] fn deref_mut(&mut self) -> &mut T { self.valu }
}

impl<'a, T: ?Sized, Lock: raw::Lock> Drop for GuardMut<'a, T, Lock> {
    #[inline] fn drop(&mut self) { self.lock.unlock(true) }
}
