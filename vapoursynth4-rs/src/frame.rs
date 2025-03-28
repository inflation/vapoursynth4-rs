/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::{api::Api, ffi, map::MapRef};

mod context;
mod format;

pub use context::*;
pub use format::*;

pub trait Frame: Sized + Send + internal::FrameFromPtr {
    fn api(&self) -> Api;

    #[must_use]
    fn as_ptr(&self) -> *mut ffi::VSFrame;

    #[must_use]
    #[inline]
    fn properties(&self) -> Option<MapRef> {
        unsafe {
            let ptr = (self.api().getFramePropertiesRO)(self.as_ptr());
            ptr.is_null().then_some(MapRef::from_ptr(ptr, self.api()))
        }
    }

    #[must_use]
    #[inline]
    fn properties_mut(&mut self) -> Option<MapRef> {
        unsafe {
            let ptr = (self.api().getFramePropertiesRW)(self.as_ptr());
            ptr.is_null().then_some(MapRef::from_ptr(ptr, self.api()))
        }
    }
}

pub(crate) mod internal {
    use super::{Api, AudioFrame, VideoFrame, ffi};

    pub trait FrameFromPtr {
        unsafe fn from_ptr(ptr: *const ffi::VSFrame, api: Api) -> Self;
    }

    impl FrameFromPtr for VideoFrame {
        #[inline]
        unsafe fn from_ptr(ptr: *const ffi::VSFrame, api: Api) -> Self {
            VideoFrame {
                handle: ptr.cast_mut(),
                api,
            }
        }
    }

    impl FrameFromPtr for AudioFrame {
        unsafe fn from_ptr(ptr: *const ffi::VSFrame, api: Api) -> Self {
            AudioFrame { handle: ptr, api }
        }
    }
}
use internal::FrameFromPtr;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct VideoFrame {
    handle: *const ffi::VSFrame,
    api: Api,
}

unsafe impl Send for VideoFrame {}

impl Frame for VideoFrame {
    #[inline]
    fn api(&self) -> Api {
        self.api
    }

    #[inline]
    fn as_ptr(&self) -> *mut ffi::VSFrame {
        self.handle.cast_mut()
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
        unsafe { Self::from_ptr((self.api.addFrameRef)(self.handle), self.api) }
    }
}

impl Drop for VideoFrame {
    fn drop(&mut self) {
        unsafe { (self.api.freeFrame)(self.handle) }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AudioFrame {
    handle: *const ffi::VSFrame,
    api: Api,
}

unsafe impl Send for AudioFrame {}

impl Frame for AudioFrame {
    fn api(&self) -> Api {
        self.api
    }

    fn as_ptr(&self) -> *mut ffi::VSFrame {
        self.handle.cast_mut()
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
        unsafe { Self::from_ptr((self.api.addFrameRef)(self.handle), self.api) }
    }
}

impl Drop for AudioFrame {
    fn drop(&mut self) {
        unsafe { (self.api.freeFrame)(self.handle) }
    }
}

pub type MediaType = ffi::VSMediaType;
