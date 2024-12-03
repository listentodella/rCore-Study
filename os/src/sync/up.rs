use core::cell::{RefCell, RefMut};

// only safe on uniprocessor - only one core
// 即只在单核时使用能够保证安全
pub struct UPSafeCell<T> {
    inner: RefCell<T>,
}

// 实现Sync
unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    pub unsafe fn new(val: T) -> Self {
        Self {
            inner: RefCell::new(val),
        }
    }

    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
