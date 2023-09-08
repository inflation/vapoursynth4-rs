use std::{ffi::CStr, ptr::NonNull};

use crate::{
    api, ffi,
    frame::Frame,
    node::{Node, VideoNode},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct FrameContext {
    handle: NonNull<ffi::VSFrameContext>,
}

impl FrameContext {
    #[must_use]
    pub unsafe fn from_ptr(ptr: *const ffi::VSFrameContext) -> FrameContext {
        FrameContext {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
        }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const ffi::VSFrameContext {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn as_mut_ptr(&mut self) -> *mut ffi::VSFrameContext {
        self.handle.as_ptr()
    }

    pub fn request_frame_filter(&mut self, n: i32, node: &VideoNode) {
        unsafe {
            (api().requestFrameFilter)(n, node.as_ptr().cast_mut(), self.as_mut_ptr());
        }
    }

    pub fn release_frame_early(&mut self, n: i32, node: &VideoNode) {
        unsafe {
            (api().releaseFrameEarly)(node.as_ptr().cast_mut(), n, self.as_mut_ptr());
        }
    }

    pub fn cache_frame(&mut self, frame: &impl Frame, n: i32) {
        unsafe {
            (api().cacheFrame)(frame.as_ptr(), n, self.as_mut_ptr());
        }
    }

    pub fn set_filter_error(&mut self, msg: &CStr) {
        unsafe {
            (api().setFilterError)(msg.as_ptr().cast(), self.as_mut_ptr());
        }
    }
}
