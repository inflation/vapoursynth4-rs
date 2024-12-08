/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::ptr::NonNull;

use crate::{api::Api, ffi, map::MapRef};

mod context;
mod format;

pub use context::*;
pub use format::*;

pub trait Frame: Sized + internal::FrameFromPtr {
    #[doc(hidden)]
    fn api(&self) -> Api;

    #[must_use]
    fn as_ptr(&self) -> *mut ffi::VSFrame;

    #[must_use]
    #[inline]
    fn properties(&self) -> Option<MapRef> {
        unsafe {
            let ptr = (self.api().getFramePropertiesRO)(self.as_ptr());
            NonNull::new(ptr.cast_mut()).map(|x| MapRef::from_ptr(x.as_ptr(), self.api()))
        }
    }

    #[must_use]
    #[inline]
    fn properties_mut(&mut self) -> Option<MapRef> {
        unsafe {
            let ptr = (self.api().getFramePropertiesRW)(self.as_ptr());
            NonNull::new(ptr).map(|x| MapRef::from_ptr(x.as_ptr(), self.api()))
        }
    }
}

pub(crate) mod internal {
    use crate::api::Api;

    use super::ffi;

    pub trait FrameFromPtr {
        unsafe fn from_ptr(ptr: *const ffi::VSFrame, api: Api) -> Self;
    }
}
use internal::FrameFromPtr;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct VideoFrame {
    handle: NonNull<ffi::VSFrame>,
    api: Api,
}

impl internal::FrameFromPtr for VideoFrame {
    #[inline]
    unsafe fn from_ptr(ptr: *const ffi::VSFrame, api: Api) -> Self {
        VideoFrame {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
            api,
        }
    }
}
impl Frame for VideoFrame {
    #[inline]
    fn api(&self) -> Api {
        self.api
    }

    #[inline]
    fn as_ptr(&self) -> *mut ffi::VSFrame {
        self.handle.as_ptr()
    }
}

impl VideoFrame {
    #[must_use]
    pub fn stride(&self, plane: i32) -> isize {
        unsafe { (self.api.getStride)(self.as_ptr(), plane) }
    }

    #[must_use]
    pub fn plane(&self, plane: i32) -> *const u8 {
        unsafe { (self.api.getReadPtr)(self.as_ptr(), plane) }
    }

    #[must_use]
    pub fn plane_mut(&mut self, plane: i32) -> *mut u8 {
        unsafe { (self.api.getWritePtr)(self.as_ptr(), plane) }
    }

    #[must_use]
    pub fn get_video_format(&self) -> &VideoFormat {
        // safety: `vf` is valid if the node is a video node
        unsafe { &*(self.api.getVideoFrameFormat)(self.as_ptr()) }
    }

    #[must_use]
    pub fn get_audio_format(&self) -> &AudioFormat {
        // safety: `af` is valid if the node is an audio node
        unsafe { &*(self.api.getAudioFrameFormat)(self.as_ptr()) }
    }

    #[must_use]
    pub fn get_type(&self) -> MediaType {
        unsafe { (self.api.getFrameType)(self.as_ptr()) }
    }

    #[must_use]
    pub fn frame_width(&self, plane: i32) -> i32 {
        unsafe { (self.api.getFrameWidth)(self.as_ptr(), plane) }
    }

    #[must_use]
    pub fn frame_height(&self, plane: i32) -> i32 {
        unsafe { (self.api.getFrameHeight)(self.as_ptr(), plane) }
    }
}

impl Clone for VideoFrame {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((self.api.addFrameRef)(self.handle.as_ptr()), self.api) }
    }
}

impl Drop for VideoFrame {
    fn drop(&mut self) {
        unsafe { (self.api.freeFrame)(self.handle.as_ptr()) }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AudioFrame {
    handle: NonNull<ffi::VSFrame>,
    api: Api,
}

impl internal::FrameFromPtr for AudioFrame {
    unsafe fn from_ptr(ptr: *const ffi::VSFrame, api: Api) -> Self {
        AudioFrame {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
            api,
        }
    }
}
impl Frame for AudioFrame {
    fn api(&self) -> Api {
        self.api
    }

    fn as_ptr(&self) -> *mut ffi::VSFrame {
        self.handle.as_ptr()
    }
}

impl AudioFrame {
    #[must_use]
    pub fn channel(&self, channel: i32) -> *const u8 {
        unsafe { (self.api.getReadPtr)(self.as_ptr(), channel) }
    }

    #[must_use]
    pub fn channel_mut(&mut self, channel: i32) -> *mut u8 {
        unsafe { (self.api.getWritePtr)(self.as_ptr(), channel) }
    }

    #[must_use]
    pub fn frame_length(&self) -> i32 {
        unsafe { (self.api.getFrameLength)(self.as_ptr()) }
    }
}

impl Clone for AudioFrame {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((self.api.addFrameRef)(self.handle.as_ptr()), self.api) }
    }
}

impl Drop for AudioFrame {
    fn drop(&mut self) {
        unsafe { (self.api.freeFrame)(self.handle.as_ptr()) }
    }
}

pub type MediaType = ffi::VSMediaType;
