/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::ptr::NonNull;

use crate::{
    api::api,
    ffi,
    map::{MapMut, MapRef},
};

mod context;
mod format;

pub use context::*;
pub use format::*;

pub trait Frame: Sized + internal::FrameFromPtr {
    #[must_use]
    fn as_ptr(&self) -> *const ffi::VSFrame;

    #[must_use]
    fn as_mut_ptr(&mut self) -> *mut ffi::VSFrame;

    #[must_use]
    fn properties(&self) -> Option<MapRef<'_>> {
        unsafe {
            let ptr = (api().getFramePropertiesRO)(self.as_ptr());
            NonNull::new(ptr.cast_mut()).map(MapRef::new)
        }
    }

    #[must_use]
    fn properties_mut(&mut self) -> Option<MapMut<'_>> {
        unsafe {
            let ptr = (api().getFramePropertiesRW)(self.as_mut_ptr());
            NonNull::new(ptr).map(MapMut::new)
        }
    }
}

pub(crate) mod internal {
    use super::ffi;

    pub trait FrameFromPtr {
        unsafe fn from_ptr(ptr: *const ffi::VSFrame) -> Self;
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct VideoFrame {
    handle: NonNull<ffi::VSFrame>,
}

impl internal::FrameFromPtr for VideoFrame {
    unsafe fn from_ptr(ptr: *const ffi::VSFrame) -> Self {
        VideoFrame {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
        }
    }
}
impl Frame for VideoFrame {
    fn as_ptr(&self) -> *const ffi::VSFrame {
        self.handle.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut ffi::VSFrame {
        self.handle.as_ptr()
    }
}

impl VideoFrame {
    pub(crate) unsafe fn from_ptr(ptr: *const ffi::VSFrame) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
        }
    }

    #[must_use]
    pub fn stride(&self, plane: i32) -> isize {
        unsafe { (api().getStride)(self.as_ptr(), plane) }
    }

    #[must_use]
    pub fn plane(&self, plane: i32) -> *const u8 {
        unsafe { (api().getReadPtr)(self.as_ptr(), plane) }
    }

    #[must_use]
    pub fn plane_mut(&mut self, plane: i32) -> *mut u8 {
        unsafe { (api().getWritePtr)(self.as_mut_ptr(), plane) }
    }

    #[must_use]
    pub fn get_video_format(&self) -> &VideoFormat {
        // safety: `vf` is valid if the node is a video node
        unsafe { &*(api().getVideoFrameFormat)(self.as_ptr()) }
    }

    #[must_use]
    pub fn get_audio_format(&self) -> &AudioFormat {
        // safety: `af` is valid if the node is an audio node
        unsafe { &*(api().getAudioFrameFormat)(self.as_ptr()) }
    }

    #[must_use]
    pub fn get_type(&self) -> MediaType {
        unsafe { (api().getFrameType)(self.as_ptr()) }
    }

    #[must_use]
    pub fn frame_width(&self, plane: i32) -> i32 {
        unsafe { (api().getFrameWidth)(self.as_ptr(), plane) }
    }

    #[must_use]
    pub fn frame_height(&self, plane: i32) -> i32 {
        unsafe { (api().getFrameHeight)(self.as_ptr(), plane) }
    }
}

impl Clone for VideoFrame {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((api().addFrameRef)(self.handle.as_ptr())) }
    }
}

impl Drop for VideoFrame {
    fn drop(&mut self) {
        unsafe { (api().freeFrame)(self.handle.as_ptr()) }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct AudioFrame {
    handle: NonNull<ffi::VSFrame>,
}

impl internal::FrameFromPtr for AudioFrame {
    unsafe fn from_ptr(ptr: *const ffi::VSFrame) -> Self {
        AudioFrame {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
        }
    }
}
impl Frame for AudioFrame {
    fn as_ptr(&self) -> *const ffi::VSFrame {
        self.handle.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut ffi::VSFrame {
        self.handle.as_ptr()
    }
}

impl AudioFrame {
    pub(crate) unsafe fn from_ptr(ptr: *const ffi::VSFrame) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
        }
    }

    #[must_use]
    pub fn channel(&self, channel: i32) -> *const u8 {
        unsafe { (api().getReadPtr)(self.as_ptr(), channel) }
    }

    #[must_use]
    pub fn channel_mut(&mut self, channel: i32) -> *mut u8 {
        unsafe { (api().getWritePtr)(self.as_mut_ptr(), channel) }
    }

    #[must_use]
    pub fn frame_length(&self) -> i32 {
        unsafe { (api().getFrameLength)(self.as_ptr()) }
    }
}

impl Clone for AudioFrame {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((api().addFrameRef)(self.handle.as_ptr())) }
    }
}

impl Drop for AudioFrame {
    fn drop(&mut self) {
        unsafe { (api().freeFrame)(self.handle.as_ptr()) }
    }
}

pub type MediaType = ffi::VSMediaType;
