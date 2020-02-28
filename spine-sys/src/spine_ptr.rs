use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

#[derive(Clone)]
pub(crate) struct SpineMutPtr<T> {
    ptr: Arc<*mut T>,
    drop_fn: Option<unsafe extern "C" fn(*mut T)>,
}
impl<T> SpineMutPtr<T> {
    pub(crate) fn new(ptr: *mut T, drop_fn: Option<unsafe extern "C" fn(*mut T)>) -> Self {
        Self {
            ptr: Arc::new(ptr),
            drop_fn,
        }
    }

    pub(crate) fn as_ref(&self) -> &T {
        unsafe { (*self.ptr).as_ref().unwrap() }
    }
    pub(crate) fn as_mut(&mut self) -> &mut T {
        unsafe { (*self.ptr).as_mut().unwrap() }
    }

    pub(crate) fn as_ptr(&self) -> *const T {
        *self.ptr as *const _
    }
    pub(crate) fn as_mut_ptr(&self) -> *mut T {
        *self.ptr
    }
}
impl<T> Drop for SpineMutPtr<T> {
    fn drop(&mut self) {
        println!("Drop: {}", std::any::type_name::<T>());
        if let Some(ptr) = Arc::get_mut(&mut self.ptr) {
            unsafe {
                if let Some(f) = self.drop_fn.take() {
                    println!("Last ref, calling drop_fn");
                    f(*ptr)
                }
            }
        }
    }
}
impl<T> Deref for SpineMutPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl<T> DerefMut for SpineMutPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
impl<T> PartialEq for SpineMutPtr<T> {
    fn eq(&self, rhv: &Self) -> bool {
        self.ptr == rhv.ptr
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct SpinePtr<T, P> {
    ptr: *const T,
    parent: SpineMutPtr<P>,
}
impl<T, P> SpinePtr<T, P> {
    pub(crate) fn new(ptr: *mut T, parent: SpineMutPtr<P>) -> Self {
        Self { ptr, parent }
    }

    pub(crate) fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref().unwrap() }
    }
    pub(crate) fn as_ptr(&self) -> *const T {
        self.ptr as *const _
    }
}
impl<T, P> Deref for SpinePtr<T, P> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
