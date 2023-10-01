use core::{
    cell::UnsafeCell,
    fmt,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct TakeOnce<T> {
    value: UnsafeCell<T>,
    taken: AtomicBool,
}

unsafe impl<T: Send> Sync for TakeOnce<T> {}

#[derive(Debug)]
pub struct AlreadyTaken;

impl fmt::Display for AlreadyTaken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("value already taken")
    }
}

impl<T> TakeOnce<T> {
    pub const fn new(value: T) -> Self {
        TakeOnce {
            value: UnsafeCell::new(value),
            taken: AtomicBool::new(false),
        }
    }
    pub fn take(&self) -> Result<&mut T, AlreadyTaken> {
        if self.taken.swap(true, Ordering::AcqRel) {
            Err(AlreadyTaken)
        } else {
            Ok(unsafe { &mut *self.value.get() })
        }
    }
}
