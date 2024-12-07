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
    ptr::{null_mut, NonNull},
};

use crate::{
    api::{api, Api},
    ffi,
    frame::{AudioFormat, AudioFrame, Frame, VideoFormat, VideoFrame},
    function::Function,
    map::MapRef,
    node::{internal::FilterExtern, Dependencies, Filter},
    plugin::{Plugin, Plugins},
    AudioInfo, ColorFamily, SampleType, VideoInfo,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct CoreRef<'c> {
    handle: NonNull<ffi::VSCore>,
    marker: PhantomData<&'c Core>,
}

impl CoreRef<'_> {
    #[must_use]
    pub(crate) unsafe fn from_ptr(ptr: *const ffi::VSCore) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
            marker: PhantomData,
        }
    }
}

impl<'c> Deref for CoreRef<'c> {
    type Target = Core;

    fn deref(&self) -> &'c Self::Target {
        unsafe { &*std::ptr::from_ref::<CoreRef<'c>>(self).cast() }
    }
}

impl<'c> DerefMut for CoreRef<'c> {
    fn deref_mut(&mut self) -> &'c mut Self::Target {
        unsafe { &mut *std::ptr::from_mut::<CoreRef<'c>>(self).cast() }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct Core {
    handle: NonNull<ffi::VSCore>,
}

impl Core {
    unsafe fn new_with(flags: i32, vsapi: &Api) -> Self {
        let core = unsafe { (vsapi.createCore)(flags) };
        Self {
            handle: NonNull::new_unchecked(core),
        }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const ffi::VSCore {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn as_mut_ptr(&mut self) -> *mut ffi::VSCore {
        self.handle.as_ptr()
    }

    pub fn set_max_cache_size(&mut self, size: i64) {
        unsafe {
            (api().setMaxCacheSize)(size, self.as_mut_ptr());
        }
    }

    pub fn set_thread_count(&mut self, count: i32) {
        unsafe {
            (api().setThreadCount)(count, self.as_mut_ptr());
        }
    }

    #[must_use]
    pub fn get_info(&self) -> ffi::VSCoreInfo {
        unsafe {
            let mut info = MaybeUninit::uninit();
            (api().getCoreInfo)(self.as_ptr().cast_mut(), info.as_mut_ptr());
            info.assume_init()
        }
    }

    /// # Panics
    ///
    /// Panic if the `dependencies` has more item than [`i32::MAX`]
    pub fn create_video_filter<F: Filter>(
        &mut self,
        out: &mut MapRef,
        name: &CStr,
        info: &VideoInfo,
        filter: Box<F>,
        dependencies: &Dependencies,
    ) {
        debug_assert!(!out.as_ptr().is_null());
        unsafe {
            (api().createVideoFilter)(
                out.as_mut_ptr(),
                name.as_ptr(),
                info,
                F::filter_get_frame,
                Some(F::filter_free),
                F::FILTER_MODE,
                dependencies.as_ptr(),
                dependencies.len().try_into().unwrap(),
                Box::into_raw(filter).cast(),
                self.as_mut_ptr(),
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
            (api().createAudioFilter)(
                out.as_mut_ptr(),
                name.as_ptr(),
                info,
                F::filter_get_frame,
                Some(F::filter_free),
                F::FILTER_MODE,
                dependencies.as_ptr(),
                dependencies.len().try_into().unwrap(),
                Box::into_raw(filter).cast(),
                self.as_mut_ptr(),
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
            let ptr = (api().newVideoFrame)(
                format,
                width,
                height,
                prop_src.map_or(null_mut(), Frame::as_ptr),
                self.as_ptr().cast_mut(),
            );
            VideoFrame::from_ptr(ptr)
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
            let ptr = (api().newVideoFrame2)(
                format,
                width,
                height,
                plane_src.as_ptr(),
                planes.as_ptr(),
                prop_src.map_or(null_mut(), Frame::as_ptr),
                self.as_ptr().cast_mut(),
            );
            VideoFrame::from_ptr(ptr)
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
            let ptr = (api().newAudioFrame)(
                format,
                num_samples,
                prop_src.map_or(null_mut(), Frame::as_ptr),
                self.as_ptr().cast_mut(),
            );
            AudioFrame::from_ptr(ptr)
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
            let ptr = (api().newAudioFrame2)(
                format,
                num_samples,
                channel_src.as_ptr(),
                channels.as_ptr(),
                prop_src.map_or(null_mut(), Frame::as_ptr),
                self.as_ptr().cast_mut(),
            );
            AudioFrame::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn copy_frame<F: Frame>(&self, frame: &F) -> F {
        unsafe { F::from_ptr((api().copyFrame)(frame.as_ptr(), self.as_ptr().cast_mut())) }
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
            (api().queryVideoFormat)(
                format.as_mut_ptr(),
                color_family,
                sample_type,
                bits_per_sample,
                subsampling_w,
                subsampling_h,
                self.as_ptr().cast_mut(),
            );
            format.assume_init()
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
            (api().queryAudioFormat)(
                format.as_mut_ptr(),
                sample_type,
                bits_per_sample,
                channel_layout,
                self.as_ptr().cast_mut(),
            );
            format.assume_init()
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
            (api().queryVideoFormatID)(
                color_family,
                sample_type,
                bits_per_sample,
                subsampling_w,
                subsampling_h,
                self.as_ptr().cast_mut(),
            )
        }
    }

    #[must_use]
    pub fn get_video_format_by_id(&self, id: u32) -> VideoFormat {
        unsafe {
            let mut format = MaybeUninit::uninit();
            (api().getVideoFormatByID)(format.as_mut_ptr(), id, self.as_ptr().cast_mut());
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
            Function::from_ptr((api().createFunction)(
                func,
                Box::into_raw(data).cast(),
                free,
                self.as_mut_ptr(),
            ))
        }
    }

    pub fn get_plugin_by_id(&self, id: &CStr) -> Option<Plugin> {
        unsafe {
            NonNull::new((api().getPluginByID)(id.as_ptr(), self.as_ptr().cast_mut()))
                .map(Plugin::new)
        }
    }

    pub fn get_plugin_by_namespace(&self, ns: &CStr) -> Option<Plugin> {
        unsafe {
            NonNull::new((api().getPluginByNamespace)(
                ns.as_ptr(),
                self.as_ptr().cast_mut(),
            ))
            .map(Plugin::new)
        }
    }

    #[must_use]
    pub fn plugins(&self) -> Plugins<'_> {
        Plugins::new(self)
    }

    pub fn log(&mut self, level: ffi::VSMessageType, msg: &CStr) {
        unsafe {
            (api().logMessage)(level, msg.as_ptr(), self.as_mut_ptr());
        }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe {
            (api().freeCore)(self.handle.as_ptr());
        }
    }
}

#[derive(Debug, Default)]
pub struct CoreBuilder<'api> {
    flags: i32,
    max_cache_size: Option<i64>,
    thread_count: Option<i32>,
    api: Option<&'api Api>,
}

impl<'api> CoreBuilder<'api> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// # Panics
    ///
    /// Return [`ApiNotSet`] if the API is not set.
    pub fn build(&mut self) -> Core {
        let mut core = unsafe { Core::new_with(self.flags, self.api.unwrap()) };
        if let Some(size) = self.max_cache_size {
            core.set_max_cache_size(size);
        }
        if let Some(count) = self.thread_count {
            core.set_thread_count(count);
        }

        core
    }

    pub fn api(&mut self, api: &'api Api) -> &mut Self {
        self.api = Some(api);
        self
    }

    pub fn enable_graph_inspection(&mut self) -> &mut Self {
        self.flags |= ffi::VSCoreCreationFlags::EnableGraphInspection as i32;
        self
    }

    pub fn disable_auto_loading(&mut self) -> &mut Self {
        self.flags |= ffi::VSCoreCreationFlags::DisableAutoLoading as i32;
        self
    }

    pub fn disable_library_unloading(&mut self) -> &mut Self {
        self.flags |= ffi::VSCoreCreationFlags::DisableLibraryUnloading as i32;
        self
    }

    pub fn max_cache_size(&mut self, size: i64) -> &mut Self {
        self.max_cache_size = Some(size);
        self
    }

    pub fn thread_count(&mut self, count: i32) -> &mut Self {
        self.thread_count = Some(count);
        self
    }
}

#[cfg(test)]
#[cfg(feature = "link-library")]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        Api::set_default();
        let core = CoreBuilder::new()
            .api(api())
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
