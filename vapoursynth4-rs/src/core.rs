/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::{
    ffi::CStr,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Deref,
    ptr::{NonNull, null_mut},
};

use bon::bon;
use core_builder::State;

use crate::{
    AudioInfo, ColorFamily, SampleType, VideoInfo,
    api::Api,
    ffi,
    frame::{
        AudioFormat, AudioFrame, AudioFrameMut, FormatName, Frame, VideoFormat, VideoFrame,
        VideoFrameMut,
    },
    function::Function,
    map::{Map, MapMut},
    node::{Dependencies, Filter, internal::FilterExtern},
    plugin::{Plugin, Plugins},
};

/// A borrowed core supplied by VapourSynth.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct CoreRef<'c> {
    handle: CoreHandle,
    marker: PhantomData<&'c ()>,
}

impl CoreRef<'_> {
    // The core must remain alive for the lifetime of the borrow.
    #[must_use]
    pub(crate) unsafe fn from_ptr(ptr: *const ffi::VSCore, api: Api) -> Self {
        Self {
            handle: CoreHandle { handle: ptr, api },
            marker: PhantomData,
        }
    }
}

impl AsRef<CoreHandle> for CoreRef<'_> {
    fn as_ref(&self) -> &CoreHandle {
        &self.handle
    }
}

impl Deref for CoreRef<'_> {
    type Target = CoreHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Core {
    handle: CoreHandle,
}

impl Deref for Core {
    type Target = CoreHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

/// Operations available on both owned and borrowed cores.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct CoreHandle {
    handle: *const ffi::VSCore,
    api: Api,
}

impl CoreHandle {
    #[must_use]
    pub fn as_ptr(&self) -> *mut ffi::VSCore {
        self.handle.cast_mut()
    }

    #[must_use]
    pub fn get_info(&self) -> ffi::VSCoreInfo {
        unsafe {
            let mut info = MaybeUninit::uninit();
            (self.api.getCoreInfo)(self.as_ptr(), info.as_mut_ptr());
            info.assume_init()
        }
    }

    /// Same as [`get_info()`](Self::get_info), but also reports the
    /// [`ffi::VSCoreCreationFlags`] the core was created with.
    #[cfg(feature = "vs-42")]
    #[must_use]
    pub fn get_info2(&self) -> ffi::VSCoreInfo2 {
        unsafe {
            let mut info = MaybeUninit::uninit();
            (self.api.getCoreInfo2)(self.as_ptr(), info.as_mut_ptr());
            info.assume_init()
        }
    }

    /// # Panics
    ///
    /// Panic if the `dependencies` has more item than [`i32::MAX`]
    pub fn create_video_filter<F: Filter>(
        &self,
        out: &mut MapMut,
        name: &CStr,
        info: &VideoInfo,
        filter: Box<F>,
        dependencies: &Dependencies,
    ) {
        debug_assert!(!out.as_ptr().is_null());
        unsafe {
            (self.api.createVideoFilter)(
                out.as_ptr(),
                name.as_ptr(),
                info,
                F::filter_get_frame,
                Some(F::filter_free),
                F::FILTER_MODE,
                dependencies.as_ptr(),
                dependencies.len().try_into().unwrap(),
                Box::into_raw(filter).cast(),
                self.as_ptr(),
            );
        }
    }

    /// # Panics
    ///
    /// Panic if the `dependencies` has more item than [`i32::MAX`]
    pub fn create_audio_filter<F: Filter>(
        &self,
        out: &mut MapMut,
        name: &CStr,
        info: &AudioInfo,
        filter: F,
        dependencies: &Dependencies,
    ) {
        let filter = Box::new(filter);
        unsafe {
            (self.api.createAudioFilter)(
                out.as_ptr(),
                name.as_ptr(),
                info,
                F::filter_get_frame,
                Some(F::filter_free),
                F::FILTER_MODE,
                dependencies.as_ptr(),
                dependencies.len().try_into().unwrap(),
                Box::into_raw(filter).cast(),
                self.as_ptr(),
            );
        }
    }

    #[must_use]
    pub fn new_video_frame(
        &self,
        format: &VideoFormat,
        width: i32,
        height: i32,
        prop_src: Option<&VideoFrame>,
    ) -> VideoFrameMut {
        unsafe {
            let ptr = (self.api.newVideoFrame)(
                format,
                width,
                height,
                prop_src.map_or(null_mut(), |f| f.as_ptr().cast()),
                self.as_ptr(),
            );
            VideoFrameMut::from_unique_ptr(ptr, self.api)
        }
    }

    /// Calls VapourSynth's video plane assembly without validation.
    ///
    /// # Safety
    ///
    /// The destination format and size must be valid. Both arrays must contain at
    /// least `format.num_planes` entries. Each non-null source must be a live
    /// video frame with a valid plane index, matching dimensions and sample
    /// storage, and compatible residency and device. Frames must be valid for
    /// this core's API.
    #[must_use]
    pub unsafe fn new_video_frame2(
        &self,
        format: &VideoFormat,
        width: i32,
        height: i32,
        plane_src: &[*const ffi::VSFrame],
        planes: &[i32],
        prop_src: Option<&VideoFrame>,
    ) -> VideoFrameMut {
        unsafe {
            let ptr = (self.api.newVideoFrame2)(
                format,
                width,
                height,
                plane_src.as_ptr(),
                planes.as_ptr(),
                prop_src.map_or(null_mut(), |f| f.as_ptr().cast()),
                self.as_ptr(),
            );
            VideoFrameMut::from_unique_ptr(ptr, self.api)
        }
    }

    #[must_use]
    pub fn new_audio_frame(
        &self,
        format: &AudioFormat,
        num_samples: i32,
        prop_src: Option<&AudioFrame>,
    ) -> AudioFrameMut {
        unsafe {
            let ptr = (self.api.newAudioFrame)(
                format,
                num_samples,
                prop_src.map_or(null_mut(), |f| f.as_ptr().cast()),
                self.as_ptr(),
            );
            AudioFrameMut::from_unique_ptr(ptr, self.api)
        }
    }

    /// Calls VapourSynth's audio channel assembly without validation.
    ///
    /// # Safety
    ///
    /// The destination format and sample count must be valid. Both arrays must
    /// contain at least `format.num_channels` entries. Each non-null source must
    /// be a live audio frame with a valid channel index and matching sample
    /// storage and frame length. Frames must be valid for this core's API.
    #[must_use]
    pub unsafe fn new_audio_frame2(
        &self,
        format: &AudioFormat,
        num_samples: i32,
        channel_src: &[*const ffi::VSFrame],
        channels: &[i32],
        prop_src: Option<&AudioFrame>,
    ) -> AudioFrameMut {
        unsafe {
            let ptr = (self.api.newAudioFrame2)(
                format,
                num_samples,
                channel_src.as_ptr(),
                channels.as_ptr(),
                prop_src.map_or(null_mut(), |f| f.as_ptr().cast()),
                self.as_ptr(),
            );
            AudioFrameMut::from_unique_ptr(ptr, self.api)
        }
    }

    #[must_use]
    pub fn copy_frame<F: Frame>(&self, frame: &F) -> F::Writable {
        unsafe {
            F::writable_from_ptr(
                (self.api.copyFrame)(frame.as_ptr(), self.as_ptr()),
                self.api,
            )
        }
    }

    /// Returns the format if VapourSynth accepts the requested parameters.
    #[must_use]
    pub fn query_video_format(
        &self,
        color_family: ColorFamily,
        sample_type: SampleType,
        bits_per_sample: i32,
        subsampling_w: i32,
        subsampling_h: i32,
    ) -> Option<VideoFormat> {
        unsafe {
            let mut format = MaybeUninit::uninit();
            let success = (self.api.queryVideoFormat)(
                format.as_mut_ptr(),
                color_family,
                sample_type,
                bits_per_sample,
                subsampling_w,
                subsampling_h,
                self.as_ptr(),
            );
            (success != 0).then(|| format.assume_init())
        }
    }

    #[must_use]
    pub fn get_video_format_name(&self, format: &VideoFormat) -> Option<String> {
        let mut buffer = FormatName::new();
        if 0 == unsafe { (self.api.getVideoFormatName)(format, buffer.as_mut_ptr().cast()) } {
            None
        } else {
            Some(buffer.to_string())
        }
    }

    /// Returns the format if VapourSynth accepts the requested parameters.
    #[must_use]
    pub fn query_audio_format(
        &self,
        sample_type: SampleType,
        bits_per_sample: i32,
        channel_layout: u64,
    ) -> Option<AudioFormat> {
        unsafe {
            let mut format = MaybeUninit::uninit();
            let success = (self.api.queryAudioFormat)(
                format.as_mut_ptr(),
                sample_type,
                bits_per_sample,
                channel_layout,
                self.as_ptr(),
            );
            (success != 0).then(|| format.assume_init())
        }
    }

    #[must_use]
    pub fn get_audio_format_name(&self, format: &AudioFormat) -> Option<String> {
        let mut buffer = FormatName::new();
        if 0 == unsafe { (self.api.getAudioFormatName)(format, buffer.as_mut_ptr().cast()) } {
            None
        } else {
            Some(buffer.to_string())
        }
    }

    #[must_use]
    pub fn query_video_format_id(
        &self,
        color_family: ColorFamily,
        sample_type: SampleType,
        bits_per_sample: i32,
        subsampling_w: i32,
        subsampling_h: i32,
    ) -> u32 {
        unsafe {
            (self.api.queryVideoFormatID)(
                color_family,
                sample_type,
                bits_per_sample,
                subsampling_w,
                subsampling_h,
                self.as_ptr(),
            )
        }
    }

    /// Returns the format if VapourSynth recognizes the ID.
    #[must_use]
    pub fn get_video_format_by_id(&self, id: u32) -> Option<VideoFormat> {
        unsafe {
            let mut format = MaybeUninit::uninit();
            let success = (self.api.getVideoFormatByID)(format.as_mut_ptr(), id, self.as_ptr());
            (success != 0).then(|| format.assume_init())
        }
    }

    pub fn create_function<T>(
        &self,
        func: ffi::VSPublicFunction,
        data: Box<T>,
        free: ffi::VSFreeFunctionData,
    ) -> Function {
        unsafe {
            Function::from_ptr(
                (self.api.createFunction)(func, Box::into_raw(data).cast(), free, self.as_ptr()),
                self.api,
            )
        }
    }

    #[must_use]
    pub fn get_plugin_by_id(&self, id: &CStr) -> Option<Plugin> {
        unsafe {
            NonNull::new((self.api.getPluginByID)(id.as_ptr(), self.as_ptr()))
                .map(|p| Plugin::new(p, self.api))
        }
    }

    #[must_use]
    pub fn get_plugin_by_namespace(&self, ns: &CStr) -> Option<Plugin> {
        unsafe {
            NonNull::new((self.api.getPluginByNamespace)(ns.as_ptr(), self.as_ptr()))
                .map(|p| Plugin::new(p, self.api))
        }
    }

    #[must_use]
    pub fn plugins(&self) -> Plugins<'_> {
        Plugins::new(self)
    }

    pub fn log(&self, level: ffi::VSMessageType, msg: &CStr) {
        unsafe {
            (self.api.logMessage)(level, msg.as_ptr(), self.as_ptr());
        }
    }
    #[must_use]
    pub fn api(&self) -> Api {
        self.api
    }

    #[must_use]
    pub fn create_map(&self) -> Map {
        unsafe {
            let ptr = (self.api.createMap)();
            Map::from_ptr(ptr, self.api)
        }
    }
}

impl Core {
    unsafe fn new_with(flags: i32, api: Api) -> Self {
        let handle = unsafe { (api.createCore)(flags) };
        Self {
            handle: CoreHandle { handle, api },
        }
    }

    pub fn set_max_cache_size(&mut self, size: i64) {
        unsafe {
            (self.api.setMaxCacheSize)(size, self.as_ptr());
        }
    }

    pub fn set_thread_count(&mut self, count: i32) {
        unsafe {
            (self.api.setThreadCount)(count, self.as_ptr());
        }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe { (self.api.freeCore)(self.as_ptr()) }
    }
}

// MARK: Builder

#[bon]
impl Core {
    #[builder]
    pub fn new(
        #[builder(field)] flags: i32,
        max_cache_size: Option<i64>,
        thread_count: Option<i32>,
        #[cfg(feature = "link-vs")]
        #[builder(default)]
        api: Api,
        #[cfg(not(feature = "link-vs"))] api: Api,
    ) -> Self {
        let mut core = unsafe { Core::new_with(flags, api) };
        if let Some(size) = max_cache_size {
            core.set_max_cache_size(size);
        }
        if let Some(count) = thread_count {
            core.set_thread_count(count);
        }

        core
    }
}

impl<S: State> CoreBuilder<S> {
    pub fn enable_graph_inspection(mut self) -> Self {
        self.flags |= ffi::VSCoreCreationFlags::EnableGraphInspection as i32;
        self
    }

    pub fn disable_auto_loading(mut self) -> Self {
        self.flags |= ffi::VSCoreCreationFlags::DisableAutoLoading as i32;
        self
    }

    pub fn disable_library_unloading(mut self) -> Self {
        self.flags |= ffi::VSCoreCreationFlags::DisableLibraryUnloading as i32;
        self
    }

    /// Log a list of all allocated frames after every completed external frame request.
    #[cfg(feature = "vs-42")]
    pub fn enable_frame_ref_debug(mut self) -> Self {
        self.flags |= ffi::VSCoreCreationFlags::EnableFrameRefDebug as i32;
        self
    }
}

#[cfg(test)]
#[cfg(feature = "link-vs")]
mod tests {
    use super::*;

    #[test]
    fn format_queries_return_none_on_failure() {
        let core = Core::builder().disable_auto_loading().build();
        assert!(
            core.query_video_format(ColorFamily::Gray, SampleType::Integer, 0, 0, 0)
                .is_none()
        );
        assert!(
            core.query_audio_format(SampleType::Integer, 16, 0)
                .is_none()
        );
        assert!(core.get_video_format_by_id(u32::MAX).is_none());
    }

    #[test]
    fn borrowed_core_leaves_owner_alive() {
        let core = Core::builder()
            .disable_auto_loading()
            .thread_count(2)
            .build();
        {
            let borrowed = unsafe { CoreRef::from_ptr(core.as_ptr(), core.api()) };
            assert_eq!(borrowed.as_ptr(), core.as_ptr());
            assert_eq!(borrowed.get_info().num_threads, 2);
        }
        assert_eq!(core.get_info().num_threads, 2);
    }

    #[test]
    fn builder() {
        let api = Api::default();
        let core = Core::builder()
            .api(api)
            .enable_graph_inspection()
            .disable_auto_loading()
            .disable_library_unloading()
            .max_cache_size(1024)
            .thread_count(4)
            .build();
        assert_eq!(core.get_info().max_framebuffer_size, 1024);
        assert_eq!(core.get_info().num_threads, 4);
    }

    #[cfg(feature = "vs-42")]
    #[test]
    fn info2_reports_creation_flags() {
        let core = Core::builder()
            .api(Api::default())
            .enable_graph_inspection()
            .disable_auto_loading()
            .build();

        let info = core.get_info2();
        let expected = ffi::VSCoreCreationFlags::EnableGraphInspection
            | ffi::VSCoreCreationFlags::DisableAutoLoading;
        assert_eq!(info.creation_flags, expected);
        assert_eq!(info.core, core.get_info().core);
    }
}
