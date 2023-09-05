use std::ptr::NonNull;

use crate::{api, ffi, Map};

#[derive(PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct FunctionRef {
    handle: NonNull<ffi::VSFunction>,
}

impl FunctionRef {
    pub(crate) unsafe fn from_ptr(ptr: *mut ffi::VSFunction) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr),
        }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *mut ffi::VSFunction {
        self.handle.as_ptr()
    }

    pub fn call(&mut self, in_: &Map, out: &mut Map) {
        unsafe {
            (api().callFunction)(self.handle.as_ptr(), in_.as_ptr(), out.as_mut_ptr());
        }
    }
}

impl Drop for FunctionRef {
    fn drop(&mut self) {
        unsafe { (api().freeFunction)(self.as_ptr()) }
    }
}

impl Clone for FunctionRef {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((api().addFunctionRef)(self.as_ptr())) }
    }
}
