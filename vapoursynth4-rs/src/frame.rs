/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::ptr::NonNull;

use crate::{api, ffi, MapMut, MapRef, MediaType};

mod context;
mod format;

pub use context::*;
pub use format::*;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct FrameRef {
    handle: NonNull<ffi::VSFrame>,
}

impl FrameRef {
    #[must_use]
    pub unsafe fn from_ptr(ptr: *const ffi::VSFrame) -> FrameRef {
        FrameRef {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
        }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const ffi::VSFrame {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn as_mut_ptr(&mut self) -> *mut ffi::VSFrame {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn properties(&self) -> MapRef<'_> {
        unsafe {
            let ptr = (api().getFramePropertiesRO)(self.as_ptr());
            MapRef::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn properties_mut(&mut self) -> MapMut<'_> {
        unsafe {
            let ptr = (api().getFramePropertiesRW)(self.as_mut_ptr());
            MapMut::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn stride(&self, plane: i32) -> Option<isize> {
        match unsafe { (api().getStride)(self.as_ptr(), plane) } {
            0 => None,
            x => Some(x),
        }
    }

    #[must_use]
    pub fn plane(&self, plane: i32) -> *const u8 {
        unsafe { (api().getReadPtr)(self.as_ptr(), plane) }
    }

    #[must_use]
    pub fn plane_mut(&mut self, plane: i32) -> *mut u8 {
        unsafe { (api().getWritePtr)(self.as_mut_ptr(), plane) }
    }

    /// # Safety
    ///
    /// The node must be a video node, otherwise the behaviour is undefined.
    #[must_use]
    pub unsafe fn get_video_format(&self) -> &VideoFormat {
        // `vf` is cpp reference internally (so it's always valid)
        &*(api().getVideoFrameFormat)(self.as_ptr())
    }

    /// # Safety
    ///
    /// The node must be an audio node, otherwise the behaviour is undefined.
    #[must_use]
    pub unsafe fn get_audio_format(&self) -> &AudioFormat {
        // `af` is cpp reference internally (so it's always valid)
        &*(api().getAudioFrameFormat)(self.as_ptr())
    }

    #[must_use]
    pub fn get_format(&self) -> Format {
        // Safety: `self.handle` is a valid pointer, and the type is correct
        match self.get_type() {
            MediaType::Video => Format::Video(unsafe { self.get_video_format() }),
            MediaType::Audio => Format::Audio(unsafe { self.get_audio_format() }),
        }
    }

    #[must_use]
    pub fn get_type(&self) -> MediaType {
        unsafe { (api().getFrameType)(self.as_ptr()).into() }
    }

    #[must_use]
    pub fn frame_width(&self, plane: i32) -> Option<i32> {
        match unsafe { (api().getFrameWidth)(self.as_ptr(), plane) } {
            0 => None,
            x => Some(x),
        }
    }

    #[must_use]
    pub fn frame_height(&self, plane: i32) -> Option<i32> {
        match unsafe { (api().getFrameHeight)(self.as_ptr(), plane) } {
            0 => None,
            x => Some(x),
        }
    }

    #[must_use]
    pub fn frame_length(&self) -> i32 {
        unsafe { (api().getFrameLength)(self.as_ptr()) }
    }
}

impl Clone for FrameRef {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((api().addFrameRef)(self.handle.as_ptr())) }
    }
}

impl Drop for FrameRef {
    fn drop(&mut self) {
        unsafe { (api().freeFrame)(self.handle.as_ptr()) }
    }
}
