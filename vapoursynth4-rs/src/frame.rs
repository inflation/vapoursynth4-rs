/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::{marker::PhantomData, mem::ManuallyDrop};

use crate::{
    api::Api,
    ffi,
    map::{MapMut, MapRef},
};

mod context;
mod format;

pub use context::*;
pub use format::*;

pub trait Frame: Sized + Send + internal::FrameFromPtr {
    fn api(&self) -> Api;

    #[must_use]
    fn as_ptr(&self) -> *mut ffi::VSFrame;

    /// Returns whether the data resides in host or GPU memory.
    #[cfg(feature = "vs-43")]
    #[must_use]
    fn residency(&self) -> ffi::VSNodeResidency;

    #[must_use]
    fn properties(&self) -> Option<MapRef<'_>>;
}

pub(crate) mod internal {
    use super::{Api, ffi};

    pub trait FrameFromPtr {
        type Writable;

        unsafe fn from_ptr(ptr: *const ffi::VSFrame, api: Api) -> Self;

        unsafe fn writable_from_ptr(ptr: *mut ffi::VSFrame, api: Api) -> Self::Writable;
    }
}
use internal::FrameFromPtr;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Video {}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Audio {}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Shared {}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Writable {}

/// A video frame. The default shared state allows reads and reference-counted cloning.
pub type VideoFrame<S = Shared> = MediaFrame<Video, S>;
pub type AudioFrame<S = Shared> = MediaFrame<Audio, S>;

/// A uniquely owned video frame. Call `freeze` before publishing it.
pub type VideoFrameMut = VideoFrame<Writable>;
/// A uniquely owned audio frame. Call `freeze` before publishing it.
pub type AudioFrameMut = AudioFrame<Writable>;

/// An owned native frame reference. Only the writable state permits mutation.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct MediaFrame<M, S = Shared> {
    handle: *const ffi::VSFrame,
    api: Api,
    marker: PhantomData<fn() -> (M, S)>,
}

unsafe impl<M, S> Send for MediaFrame<M, S> {}

impl<M, S> MediaFrame<M, S> {
    fn new(handle: *const ffi::VSFrame, api: Api) -> Self {
        assert!(!handle.is_null(), "VapourSynth returned a null frame");
        Self {
            handle,
            api,
            marker: PhantomData,
        }
    }

    #[must_use]
    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::VSFrame {
        self.handle.cast_mut()
    }

    #[cfg(feature = "vs-43")]
    #[must_use]
    #[inline]
    pub fn residency(&self) -> ffi::VSNodeResidency {
        unsafe { (self.api.getFrameResidency)(self.handle) }
    }

    #[must_use]
    #[inline]
    pub fn properties(&self) -> Option<MapRef<'_>> {
        unsafe {
            let ptr = (self.api.getFramePropertiesRO)(self.handle);
            (!ptr.is_null()).then(|| MapRef::from_ptr(ptr, self.api))
        }
    }
}

impl<M> FrameFromPtr for MediaFrame<M> {
    type Writable = MediaFrame<M, Writable>;

    unsafe fn from_ptr(ptr: *const ffi::VSFrame, api: Api) -> Self {
        Self::new(ptr, api)
    }

    unsafe fn writable_from_ptr(ptr: *mut ffi::VSFrame, api: Api) -> Self::Writable {
        unsafe { MediaFrame::from_unique_ptr(ptr, api) }
    }
}

impl<M> Frame for MediaFrame<M> {
    #[inline]
    fn api(&self) -> Api {
        self.api
    }

    #[inline]
    fn as_ptr(&self) -> *mut ffi::VSFrame {
        self.as_ptr()
    }

    #[cfg(feature = "vs-43")]
    fn residency(&self) -> ffi::VSNodeResidency {
        self.residency()
    }

    fn properties(&self) -> Option<MapRef<'_>> {
        self.properties()
    }
}

impl<M> MediaFrame<M, Writable> {
    // The caller transfers the only reference to a newly allocated or copied frame.
    pub(crate) unsafe fn from_unique_ptr(ptr: *mut ffi::VSFrame, api: Api) -> Self {
        Self::new(ptr, api)
    }

    /// Publishes the frame without copying it or changing its reference count.
    #[must_use]
    #[inline]
    pub fn freeze(self) -> MediaFrame<M> {
        let frame = ManuallyDrop::new(self);
        MediaFrame {
            handle: frame.handle,
            api: frame.api,
            marker: PhantomData,
        }
    }

    #[must_use]
    #[inline]
    pub fn properties_mut(&mut self) -> Option<MapMut<'_>> {
        unsafe {
            let ptr = (self.api.getFramePropertiesRW)(self.as_ptr());
            (!ptr.is_null()).then(|| MapMut::from_ptr(ptr, self.api))
        }
    }
}

impl<S> VideoFrame<S> {
    #[must_use]
    pub fn stride(&self, plane: i32) -> isize {
        unsafe { (self.api.getStride)(self.handle, plane) }
    }

    #[must_use]
    pub fn plane(&self, plane: i32) -> *const u8 {
        unsafe { (self.api.getReadPtr)(self.handle, plane) }
    }

    #[must_use]
    pub fn get_video_format(&self) -> &VideoFormat {
        unsafe { &*(self.api.getVideoFrameFormat)(self.handle) }
    }

    #[must_use]
    pub fn get_type(&self) -> MediaType {
        unsafe { (self.api.getFrameType)(self.handle) }
    }

    #[must_use]
    pub fn frame_width(&self, plane: i32) -> i32 {
        unsafe { (self.api.getFrameWidth)(self.handle, plane) }
    }

    #[must_use]
    pub fn frame_height(&self, plane: i32) -> i32 {
        unsafe { (self.api.getFrameHeight)(self.handle, plane) }
    }
}

impl VideoFrameMut {
    #[must_use]
    pub fn plane_mut(&mut self, plane: i32) -> *mut u8 {
        unsafe { (self.api.getWritePtr)(self.as_ptr(), plane) }
    }
}

impl<S> AudioFrame<S> {
    #[must_use]
    pub fn channel(&self, channel: i32) -> *const u8 {
        unsafe { (self.api.getReadPtr)(self.handle, channel) }
    }

    #[must_use]
    pub fn get_audio_format(&self) -> &AudioFormat {
        unsafe { &*(self.api.getAudioFrameFormat)(self.handle) }
    }

    #[must_use]
    pub fn frame_length(&self) -> i32 {
        unsafe { (self.api.getFrameLength)(self.handle) }
    }
}

impl AudioFrameMut {
    #[must_use]
    pub fn channel_mut(&mut self, channel: i32) -> *mut u8 {
        unsafe { (self.api.getWritePtr)(self.as_ptr(), channel) }
    }
}

impl<M> Clone for MediaFrame<M> {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((self.api.addFrameRef)(self.handle), self.api) }
    }
}

impl<M, S> Drop for MediaFrame<M, S> {
    fn drop(&mut self) {
        unsafe { (self.api.freeFrame)(self.handle) }
    }
}

pub type MediaType = ffi::VSMediaType;

#[cfg(all(test, feature = "link-vs"))]
mod tests {
    use crate::{
        ColorFamily, SampleType,
        core::Core,
        key,
        map::{AppendMode, Value},
    };

    #[test]
    fn assembling_frames_retains_source_samples() {
        let core = crate::core::Core::builder().disable_auto_loading().build();
        let gray = core
            .query_video_format(
                crate::ffi::VSColorFamily::Gray,
                crate::ffi::VSSampleType::Integer,
                8,
                0,
                0,
            )
            .unwrap();
        let mut source = core.new_video_frame(&gray, 16, 16, None);
        unsafe { source.plane_mut(0).write(42) };
        let source = source.freeze();
        let video = unsafe {
            core.new_video_frame2(&gray, 16, 16, &[source.as_ptr()], &[0], Some(&source))
        };
        drop(source);
        let video = video.freeze();
        assert_eq!(video.get_video_format().bits_per_sample, 8);
        assert!(video.stride(0) >= 16);
        assert_eq!(unsafe { video.plane(0).read() }, 42);

        let stereo = core
            .query_audio_format(crate::ffi::VSSampleType::Integer, 16, 3)
            .unwrap();
        let mut source = core.new_audio_frame(&stereo, 32, None);
        unsafe {
            source
                .channel_mut(0)
                .copy_from_nonoverlapping(17_i16.to_ne_bytes().as_ptr(), 2);
            source
                .channel_mut(1)
                .copy_from_nonoverlapping(29_i16.to_ne_bytes().as_ptr(), 2);
        }
        let source = source.freeze();
        let audio = unsafe {
            core.new_audio_frame2(
                &stereo,
                32,
                &[source.as_ptr(), source.as_ptr()],
                &[1, 0],
                Some(&source),
            )
        };
        drop(source);
        let audio = audio.freeze();
        assert_eq!(audio.get_audio_format().num_channels, 2);
        assert_eq!(audio.frame_length(), 32);
        assert_eq!(unsafe { audio.channel(0).read() }, 29_i16.to_ne_bytes()[0]);
        assert_eq!(unsafe { audio.channel(1).read() }, 17_i16.to_ne_bytes()[0]);
    }

    #[test]
    fn freezing_preserves_handle_and_copying_isolates_properties() {
        let core = Core::builder().build();
        let format = core
            .query_video_format(ColorFamily::Gray, SampleType::Integer, 8, 0, 0)
            .unwrap();
        let mut original = core.new_video_frame(&format, 16, 16, None);
        let key = key!(c"test");
        original
            .properties_mut()
            .unwrap()
            .set(key, Value::Int(1), AppendMode::Replace)
            .unwrap();

        let ptr = original.as_ptr();
        let original = original.freeze();
        assert_eq!(original.as_ptr(), ptr);
        let alias = original.clone();
        let mut copy = core.copy_frame(&original);
        copy.properties_mut()
            .unwrap()
            .set(key, Value::Int(2), AppendMode::Replace)
            .unwrap();

        assert_eq!(alias.properties().unwrap().get_int(key, 0), Ok(1));
        assert_eq!(copy.freeze().properties().unwrap().get_int(key, 0), Ok(2));
    }
}
