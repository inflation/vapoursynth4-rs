use std::ptr::NonNull;

use vapoursynth4_sys as ffi;

use crate::{api, Core, Map};

pub struct FunctionRef {
    handle: NonNull<ffi::VSFunction>,
}

impl FunctionRef {
    pub fn new<T>(
        func: ffi::VSPublicFunction,
        data: &mut T,
        free: ffi::VSFreeFunctionData,
        core: &Core,
    ) -> Self {
        let handle = unsafe {
            NonNull::new_unchecked((api().createFunction)(
                func,
                (data as *mut T).cast(),
                free,
                core.as_ptr(),
            ))
        };

        Self { handle }
    }

    pub fn call(&mut self, in_: &Map, out: &mut Map) {
        unsafe {
            (api().callFunction)(self.handle.as_ptr(), in_.as_ptr(), out.as_mut_ptr());
        }
    }

    pub(crate) unsafe fn from_raw(ptr: *mut ffi::VSFunction) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr),
        }
    }

    pub fn as_mut_ptr(&self) -> *mut ffi::VSFunction {
        self.handle.as_ptr()
    }
}

impl Drop for FunctionRef {
    fn drop(&mut self) {
        unsafe { (api().freeFunction)(self.handle.as_ptr()) }
    }
}

impl Clone for FunctionRef {
    fn clone(&self) -> Self {
        let handle =
            unsafe { NonNull::new_unchecked((api().addFunctionRef)(self.handle.as_ptr())) };

        Self { handle }
    }
}
