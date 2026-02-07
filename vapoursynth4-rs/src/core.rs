/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::{
    ffi::CStr,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::{NonNull, null_mut},
};

use bon::bon;
use core_builder::State;

use crate::{
    AudioInfo, ColorFamily, SampleType, VideoInfo,
    api::Api,
    ffi,
    frame::{
        AudioFormat, AudioFrame, FormatName, Frame, VideoFormat, VideoFrame, internal::FrameFromPtr,
    },
    function::Function,
    map::{Map, MapRef},
    node::{Dependencies, Filter, internal::FilterExtern},
    plugin::{Plugin, Plugins},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CoreRef<'c> {
    handle: *const ffi::VSCore,
    api: Api,
    marker: PhantomData<&'c ()>,
}

impl CoreRef<'_> {
    #[must_use]
    pub(crate) unsafe fn from_ptr(ptr: *const ffi::VSCore, api: Api) -> Self {
        Self {
            handle: ptr.cast_mut(),
            api,
            marker: PhantomData,
        }
    }
}

impl AsRef<Core> for CoreRef<'_> {
    fn as_ref(&self) -> &Core {
        unsafe { &*std::ptr::from_ref(self).cast() }
    }
}

impl Deref for CoreRef<'_> {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        unsafe { &*std::ptr::from_ref(self).cast() }
    }
}

impl DerefMut for CoreRef<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *std::ptr::from_mut(self).cast() }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Core {
    handle: *const ffi::VSCore,
    api: Api,
}

impl Core {
    #[must_use]
    pub fn as_ptr(&self) -> *mut ffi::VSCore {
        self.handle.cast_mut()
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

    #[must_use]
    pub fn get_info(&self) -> ffi::VSCoreInfo {
        unsafe {
            let mut info = MaybeUninit::uninit();
            (self.api.getCoreInfo)(self.as_ptr(), info.as_mut_ptr());
            info.assume_init()
        }
    }

    /// # Panics
    ///
    /// Panic if the `dependencies` has more item than [`i32::MAX`]
    pub fn create_video_filter<F: Filter>(
        &mut self,
        out: MapRef,
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
        &mut self,
        out: &mut MapRef,
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
    ) -> VideoFrame {
        unsafe {
            let ptr = (self.api.newVideoFrame)(
                format,
                width,
                height,
                prop_src.map_or(null_mut(), |f| f.as_ptr().cast()),
                self.as_ptr(),
            );
            VideoFrame::from_ptr(ptr, self.api)
        }
    }

    #[must_use]
    pub fn new_video_frame2(
        &self,
        format: &VideoFormat,
        width: i32,
        height: i32,
        plane_src: &[*const ffi::VSFrame],
        planes: &[i32],
        prop_src: Option<&VideoFrame>,
    ) -> VideoFrame {
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
            VideoFrame::from_ptr(ptr, self.api)
        }
    }

    #[must_use]
    pub fn new_audio_frame(
        &self,
        format: &AudioFormat,
        num_samples: i32,
        prop_src: Option<&AudioFrame>,
    ) -> AudioFrame {
        unsafe {
            let ptr = (self.api.newAudioFrame)(
                format,
                num_samples,
                prop_src.map_or(null_mut(), |f| f.as_ptr().cast()),
                self.as_ptr(),
            );
            AudioFrame::from_ptr(ptr, self.api)
        }
    }

    #[must_use]
    pub fn new_audio_frame2(
        &self,
        format: &AudioFormat,
        num_samples: i32,
        channel_src: &[*const ffi::VSFrame],
        channels: &[i32],
        prop_src: Option<&AudioFrame>,
    ) -> AudioFrame {
        unsafe {
            let ptr = (self.api.newAudioFrame2)(
                format,
                num_samples,
                channel_src.as_ptr(),
                channels.as_ptr(),
                prop_src.map_or(null_mut(), |f| f.as_ptr().cast()),
                self.as_ptr(),
            );
            AudioFrame::from_ptr(ptr, self.api)
        }
    }

    #[must_use]
    pub fn copy_frame<F: Frame>(&self, frame: &F) -> F {
        unsafe {
            F::from_ptr(
                (self.api.copyFrame)(frame.as_ptr(), self.as_ptr()),
                self.api,
            )
        }
    }

    #[must_use]
    pub fn query_video_format(
        &self,
        color_family: ColorFamily,
        sample_type: SampleType,
        bits_per_sample: i32,
        subsampling_w: i32,
        subsampling_h: i32,
    ) -> VideoFormat {
        unsafe {
            let mut format = MaybeUninit::uninit();
            (self.api.queryVideoFormat)(
                format.as_mut_ptr(),
                color_family,
                sample_type,
                bits_per_sample,
                subsampling_w,
                subsampling_h,
                self.as_ptr(),
            );
            format.assume_init()
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

    #[must_use]
    pub fn query_audio_format(
        &self,
        sample_type: SampleType,
        bits_per_sample: i32,
        channel_layout: u64,
    ) -> AudioFormat {
        unsafe {
            let mut format = MaybeUninit::uninit();
            (self.api.queryAudioFormat)(
                format.as_mut_ptr(),
                sample_type,
                bits_per_sample,
                channel_layout,
                self.as_ptr(),
            );
            format.assume_init()
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

    #[must_use]
    pub fn get_video_format_by_id(&self, id: u32) -> VideoFormat {
        unsafe {
            let mut format = MaybeUninit::uninit();
            (self.api.getVideoFormatByID)(format.as_mut_ptr(), id, self.as_ptr());
            format.assume_init()
        }
    }

    pub fn create_function<T>(
        &mut self,
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

    pub fn log(&mut self, level: ffi::VSMessageType, msg: &CStr) {
        unsafe {
            (self.api.logMessage)(level, msg.as_ptr(), self.as_ptr());
        }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe {
            (self.api.freeCore)(self.handle.cast_mut());
        }
    }
}

// MARK: Helper

impl Core {
    unsafe fn new_with(flags: i32, api: Api) -> Self {
        let core = unsafe { (api.createCore)(flags) };
        Self { handle: core, api }
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
}

#[cfg(test)]
#[cfg(feature = "link-vs")]
mod tests {
    use super::*;

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
}
