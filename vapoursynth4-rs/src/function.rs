use crate::{api::Api, ffi, map::Map};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Function {
    handle: *const ffi::VSFunction,
    api: Api,
}

unsafe impl Send for Function {}

impl Function {
    pub(crate) unsafe fn from_ptr(ptr: *mut ffi::VSFunction, api: Api) -> Self {
        Self { handle: ptr, api }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *mut ffi::VSFunction {
        self.handle.cast_mut()
    }

    pub fn call(&mut self, in_: &Map, out: &mut Map) {
        unsafe {
            (self.api.callFunction)(self.as_ptr(), in_.as_ptr(), out.as_ptr());
        }
    }
}

impl Drop for Function {
    fn drop(&mut self) {
        unsafe { (self.api.freeFunction)(self.as_ptr()) }
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((self.api.addFunctionRef)(self.as_ptr()), self.api) }
    }
}
