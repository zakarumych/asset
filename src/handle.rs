
use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

struct Inner<T> {
    value: ManuallyDrop<UnsafeCell<T>>,
    ready: AtomicBool,
}

impl<T> Inner<T> {
    fn new() -> Self {
        use std::mem::uninitialized;
        Inner {
            value: unsafe { uninitialized() },
            ready: AtomicBool::new(false),
        }
    }

    fn full(value: T) -> Self {
        Inner {
            value: ManuallyDrop::new(UnsafeCell::new(value)),
            ready: AtomicBool::new(true),
        }
    }

    fn ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    unsafe fn get(&self) -> &T {
        debug_assert!(self.ready());
        &*self.value.get()
    }

    unsafe fn store(&self, value: T) {
        use std::ptr::write;
        debug_assert!(!self.ready());
        write(self.value.get(), value);
        self.ready.store(true, Ordering::Release);
    }
}

impl<T> Drop for Inner<T> {
    fn drop(&mut self) {
        use std::mem::needs_drop;
        if needs_drop::<T>() && self.ready.load(Ordering::Acquire) {
            unsafe {
                ManuallyDrop::drop(&mut self.value);
            }
        }
    }
}

/// Slot suitable for storing one value of type `T`.
pub struct Slot<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Slot<T> {
    /// Create new empty slot.
    pub fn new() -> Self {
        Slot {
            inner: Arc::new(Inner::new())
        }
    }


    /// Store value.
    /// Returns `Handle` that can be used to get reference to stored value.
    pub fn store(self, value: T) -> Handle<T> {
        unsafe {
            self.inner.store(value);
        }
        Handle {
            inner: self.inner,
        }
    }

    /// Get token for the value that will be stored.
    pub fn token(&self) -> Token<T> {
        Token {
            inner: Arc::clone(&self.inner)
        }
    }
}

/// Token represents value that will be stored at some point.
#[derive(Clone)]
pub struct Token<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Token<T> {
    /// Try to convert into `Handle`.
    /// Returns `Some` if value has be stored.
    /// Returns `None` otherwise.
    pub fn handle(&self) -> Option<Handle<T>> {
        if self.inner.ready() {
            Some(Handle {
                inner: Arc::clone(&self.inner)
            })
        } else {
            None
        }
    }
}

/// Handle to the value stored through `Slot`.
#[derive(Clone)]
pub struct Handle<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Handle<T> {
    /// Create new handle with value.
    /// This is almost identical to creating `Arc`.
    pub fn new(value: T) -> Self {
        Handle {
            inner: Arc::new(Inner::full(value)),
        }
    }
}

impl<T> Deref for Handle<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            self.inner.get()
        }
    }
}