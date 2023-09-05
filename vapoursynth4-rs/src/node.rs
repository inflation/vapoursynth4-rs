/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::{
    ffi::{CStr, CString},
    ptr::NonNull,
};

use crate::{api, ffi, AudioInfo, Core, FrameRef, MediaType, VideoInfo};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Info<'n> {
    Video(&'n VideoInfo),
    Audio(&'n AudioInfo),
}

#[derive(Debug)]
pub struct NodeRef {
    handle: NonNull<ffi::VSNode>,
}

impl NodeRef {
    #[must_use]
    pub unsafe fn from_ptr(ptr: *mut ffi::VSNode) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr),
        }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const ffi::VSNode {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn as_mut_ptr(&mut self) -> *mut ffi::VSNode {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn get_type(&self) -> MediaType {
        // Safety: `self.handle` is a valid pointer
        unsafe { (api().getNodeType)(self.as_ptr().cast_mut()) }.into()
    }

    /// # Safety
    ///
    /// The node must be a video node, otherwise the behaviour is undefined.
    #[must_use]
    pub unsafe fn get_video_info(&self) -> &VideoInfo {
        // `vi` is cpp reference internally (so it's always valid)
        &*(api().getVideoInfo)(self.as_ptr().cast_mut())
    }

    /// # Safety
    ///
    /// The node must be an audio node, otherwise the behaviour is undefined.
    #[must_use]
    pub unsafe fn get_audio_info(&self) -> &AudioInfo {
        // `ai` is cpp reference internally (so it's always valid)
        &*(api().getAudioInfo)(self.as_ptr().cast_mut())
    }

    #[must_use]
    pub fn get_info(&self) -> Info {
        // Safety: `self.handle` is a valid pointer, and the type is correct
        match self.get_type() {
            MediaType::Video => Info::Video(unsafe { self.get_video_info() }),
            MediaType::Audio => Info::Audio(unsafe { self.get_audio_info() }),
        }
    }

    /// # Panics
    ///
    /// Panics if the the dependency index is larger than [`i32::MAX`].
    pub fn new_video<F: Filter>(
        name: &str,
        info: &VideoInfo,
        filter: &mut F,
        dependencies: &[ffi::VSFilterDependency],
        core: &mut Core,
    ) -> Option<Self> {
        let name = CString::new(name).ok()?;
        let node = unsafe {
            (api().createVideoFilter2)(
                name.as_ptr(),
                info,
                filter.get_frame(),
                filter.free(),
                filter.filter_mode().into(),
                dependencies.as_ptr(),
                dependencies.len().try_into().unwrap(),
                (filter.instance_data() as *mut F::InstanceData).cast(),
                core.as_mut_ptr(),
            )
        };

        if node.is_null() {
            None
        } else {
            Some(unsafe { Self::from_ptr(node) })
        }
    }

    /// # Panics
    ///
    /// Panics if the the dependency index is larger than [`i32::MAX`].
    pub fn new_audio<F: Filter>(
        name: &str,
        info: &AudioInfo,
        filter: &mut F,
        dependencies: &[ffi::VSFilterDependency],
        core: &mut Core,
    ) -> Option<Self> {
        let name = CString::new(name).ok()?;
        let node = unsafe {
            (api().createAudioFilter2)(
                name.as_ptr(),
                info,
                filter.get_frame(),
                filter.free(),
                filter.filter_mode().into(),
                dependencies.as_ptr(),
                dependencies.len().try_into().unwrap(),
                (filter.instance_data() as *mut F::InstanceData).cast(),
                core.as_mut_ptr(),
            )
        };

        if node.is_null() {
            None
        } else {
            Some(unsafe { Self::from_ptr(node) })
        }
    }

    pub fn set_linear_filter(&mut self) -> i32 {
        unsafe { (api().setLinearFilter)(self.as_mut_ptr()) }
    }

    pub fn set_cache_mode(&mut self, mode: CacheMode) {
        unsafe { (api().setCacheMode)(self.as_mut_ptr(), mode.into()) }
    }

    pub fn set_cache_options(&mut self, fixed_size: i32, max_size: i32, max_history_size: i32) {
        unsafe {
            (api().setCacheOptions)(self.as_mut_ptr(), fixed_size, max_size, max_history_size);
        }
    }

    pub fn get_frame(&self, n: i32) -> Result<FrameRef, String> {
        let mut buf = vec![0; 1024];
        let ptr = unsafe { (api().getFrame)(n, self.as_ptr().cast_mut(), buf.as_mut_ptr(), 1024) };

        if ptr.is_null() {
            let mut buf = std::mem::ManuallyDrop::new(buf);
            Err(unsafe { CStr::from_ptr(buf.as_mut_ptr()) }
                .to_string_lossy()
                .into_owned())
        } else {
            unsafe { Ok(FrameRef::from_ptr(ptr)) }
        }
    }

    // TODO: Find a better way to handle callbacks
    pub fn get_frame_async<D, F>(&self, _n: i32, _data: &mut D)
    where
        F: Fn(D, FrameRef, i32) -> Result<(), String>,
    {
        todo!()
    }
}

impl Clone for NodeRef {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((api().addNodeRef)(self.as_ptr().cast_mut())) }
    }
}

impl Drop for NodeRef {
    fn drop(&mut self) {
        unsafe { (api().freeNode)(self.as_mut_ptr()) }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum FilterMode {
    /// Completely parallel execution. Multiple threads will call a filter's "getFrame" function,
    /// to fetch several frames in parallel.
    Parallel,
    /// For filters that are serial in nature but can request in advance one or more frames
    /// they need. A filter's "getFrame" function will be called from multiple threads at a time
    /// with activation reason [`ActivationReason::Initial`],
    /// but only one thread will call it with activation reason
    /// [`ActivationReason::AllFramesReady`] at a time.
    ParallelRequests,
    /// Only one thread can call the filter's "getFrame" function at a time.
    /// Useful for filters that modify or examine their internal state to
    /// determine which frames to request.
    ///
    /// While the "getFrame" function will only run in one thread at a time,
    /// the calls can happen in any order. For example, it can be called with reason
    /// [`ActivationReason::Initial`] for frame 0, then again with reason
    /// [`ActivationReason::Initial`] for frame 1,
    /// then with reason [`ActivationReason::AllFramesReady`]  for frame 0.
    Unordered,
    /// For compatibility with other filtering architectures.
    /// *DO NOT USE IN NEW FILTERS*. The filter's "getFrame" function only ever gets called from
    /// one thread at a time. Unlike [`FilterMode::Unordered`],
    /// only one frame is processed at a time.
    FrameState,
}

impl From<FilterMode> for ffi::VSFilterMode {
    fn from(mode: FilterMode) -> Self {
        use ffi::VSFilterMode as vm;
        use FilterMode as m;

        match mode {
            m::Parallel => vm::fmParallel,
            m::ParallelRequests => vm::fmParallelRequests,
            m::Unordered => vm::fmUnordered,
            m::FrameState => vm::fmFrameState,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum CacheMode {
    /// Cache is enabled or disabled based on the reported request patterns
    /// and number of consumers.
    Auto = -1,
    /// Never cache anything.
    ForceDisable = 0,
    /// Never cache anything.
    ForceEnable = 1,
}

impl From<CacheMode> for ffi::VSCacheMode {
    fn from(mode: CacheMode) -> Self {
        use ffi::VSCacheMode as vm;
        use CacheMode as m;

        match mode {
            m::Auto => vm::cmAuto,
            m::ForceDisable => vm::cmForceDisable,
            m::ForceEnable => vm::cmForceEnable,
        }
    }
}

pub trait Filter {
    type InstanceData;

    fn get_frame(&self) -> ffi::VSFilterGetFrame;
    fn free(&self) -> ffi::VSFilterFree;
    fn filter_mode(&self) -> FilterMode;
    fn instance_data(&mut self) -> *mut Self::InstanceData;
}

impl<T: Filter + ?Sized> Filter for Box<T> {
    type InstanceData = T::InstanceData;

    fn get_frame(&self) -> ffi::VSFilterGetFrame {
        (**self).get_frame()
    }

    fn free(&self) -> ffi::VSFilterFree {
        (**self).free()
    }

    fn filter_mode(&self) -> FilterMode {
        (**self).filter_mode()
    }

    fn instance_data(&mut self) -> *mut Self::InstanceData {
        (**self).instance_data()
    }
}
