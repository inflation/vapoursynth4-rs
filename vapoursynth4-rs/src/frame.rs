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

pub trait FrameType: private::Sealed {}
mod private {
    pub trait Sealed {}
    impl Sealed for super::FrameTypeVideo {}
    impl Sealed for super::FrameTypeAudio {}
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FrameTypeVideo;
impl FrameType for FrameTypeVideo {}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FrameTypeAudio;
impl FrameType for FrameTypeAudio {}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Frame<T: FrameType> {
    handle: NonNull<ffi::VSFrame>,
    api: Api,
    _marker: std::marker::PhantomData<T>,
}

impl<T: FrameType> Frame<T> {
    #[inline]
    pub(crate) unsafe fn from_ptr(ptr: *const ffi::VSFrame, api: Api) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
            api,
            _marker: std::marker::PhantomData,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_ptr(&self) -> *mut ffi::VSFrame {
        self.handle.as_ptr()
    }

    #[must_use]
    #[inline]
    pub fn properties(&self) -> Option<MapRef> {
        unsafe {
            let ptr = (self.api.getFramePropertiesRO)(self.as_ptr());
            NonNull::new(ptr.cast_mut()).map(|x| MapRef::from_ptr(x.as_ptr(), self.api))
        }
    }

    #[must_use]
    #[inline]
    pub fn properties_mut(&mut self) -> Option<MapRef> {
        unsafe {
            let ptr = (self.api.getFramePropertiesRW)(self.as_ptr());
            NonNull::new(ptr).map(|x| MapRef::from_ptr(x.as_ptr(), self.api))
        }
    }

    #[must_use]
    pub fn get_type(&self) -> MediaType {
        unsafe { (self.api.getFrameType)(self.as_ptr()) }
    }
}

impl Frame<FrameTypeVideo> {
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
    pub fn frame_width(&self, plane: i32) -> i32 {
        unsafe { (self.api.getFrameWidth)(self.as_ptr(), plane) }
    }

    #[must_use]
    pub fn frame_height(&self, plane: i32) -> i32 {
        unsafe { (self.api.getFrameHeight)(self.as_ptr(), plane) }
    }
}

impl Frame<FrameTypeAudio> {
    #[must_use]
    pub fn channel(&self, channel: i32) -> *const u8 {
        unsafe { (self.api.getReadPtr)(self.as_ptr(), channel) }
    }

    #[must_use]
    pub fn channel_mut(&mut self, channel: i32) -> *mut u8 {
        unsafe { (self.api.getWritePtr)(self.as_ptr(), channel) }
    }

    #[must_use]
    pub fn get_audio_format(&self) -> &AudioFormat {
        // safety: `af` is valid if the node is an audio node
        unsafe { &*(self.api.getAudioFrameFormat)(self.as_ptr()) }
    }

    #[must_use]
    pub fn frame_length(&self) -> i32 {
        unsafe { (self.api.getFrameLength)(self.as_ptr()) }
    }
}

impl<T: FrameType> Clone for Frame<T> {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((self.api.addFrameRef)(self.handle.as_ptr()), self.api) }
    }
}

impl<T: FrameType> Drop for Frame<T> {
    fn drop(&mut self) {
        unsafe { (self.api.freeFrame)(self.handle.as_ptr()) }
    }
}

pub type MediaType = ffi::VSMediaType;

pub type VideoFrame = Frame<FrameTypeVideo>;
pub type AudioFrame = Frame<FrameTypeAudio>;
