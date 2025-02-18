/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::ptr::NonNull;

use crate::{api::Api, ffi, map::Map};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Function {
    handle: NonNull<ffi::VSFunction>,
    api: Api,
}

impl Function {
    pub(crate) unsafe fn from_ptr(ptr: *mut ffi::VSFunction, api: Api) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr),
            api,
        }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *mut ffi::VSFunction {
        self.handle.as_ptr()
    }

    pub fn call(&mut self, in_: &Map, out: &mut Map) {
        unsafe {
            (self.api.callFunction)(self.handle.as_ptr(), in_.as_ptr(), out.as_ptr());
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
