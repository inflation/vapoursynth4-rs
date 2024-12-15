use std::{ffi::CStr, ptr::NonNull};

use crate::{api::Api, ffi, frame::Frame};

use super::FrameType;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FrameContext {
    handle: NonNull<ffi::VSFrameContext>,
    api: Api,
}

impl FrameContext {
    #[must_use]
    pub(crate) unsafe fn from_ptr(ptr: *const ffi::VSFrameContext, api: Api) -> FrameContext {
        FrameContext {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
            api,
        }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *mut ffi::VSFrameContext {
        self.handle.as_ptr()
    }

    pub fn cache_frame<T: FrameType>(&self, frame: &Frame<T>, n: i32) {
        unsafe {
            (self.api.cacheFrame)(frame.as_ptr(), n, self.as_ptr());
        }
    }

    pub fn set_filter_error(&self, msg: &CStr) {
        unsafe {
            (self.api.setFilterError)(msg.as_ptr().cast(), self.as_ptr());
        }
    }
}

unsafe impl Sync for FrameContext {}
unsafe impl Send for FrameContext {}
