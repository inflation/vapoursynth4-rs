/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::{ffi::CString, ptr::NonNull};

use vapoursynth4_sys as ffi;

use crate::{api, AudioInfo, Core, MediaType, VideoInfo};

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
    pub(crate) unsafe fn from_raw(ptr: *mut ffi::VSNode) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr),
        }
    }

    #[must_use]
    pub fn as_mut_ptr(&self) -> *mut ffi::VSNode {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn get_type(&self) -> MediaType {
        // Safety: `self.handle` is a valid pointer
        unsafe { (api().getNodeType)(self.handle.as_ptr()) }.into()
    }

    /// # Safety
    ///
    /// The node must be a video node, otherwise the behaviour is undefined.
    #[must_use]
    pub unsafe fn get_video_info(&self) -> &VideoInfo {
        // `vi` is cpp reference internally (so it's always valid)
        &*(api().getVideoInfo)(self.handle.as_ptr())
    }

    /// # Safety
    ///
    /// The node must be an audio node, otherwise the behaviour is undefined.
    #[must_use]
    pub unsafe fn get_audio_info(&self) -> &AudioInfo {
        // `ai` is cpp reference internally (so it's always valid)
        &*(api().getAudioInfo)(self.handle.as_ptr())
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
        core: &Core,
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
                core.as_ptr(),
            )
        };

        if node.is_null() {
            None
        } else {
            Some(unsafe { Self::from_raw(node) })
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
        core: &Core,
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
                core.as_ptr(),
            )
        };

        if node.is_null() {
            None
        } else {
            Some(unsafe { Self::from_raw(node) })
        }
    }
}

impl Clone for NodeRef {
    fn clone(&self) -> Self {
        Self {
            /// Safety: `self.handle` is a valid pointer
            handle: unsafe { NonNull::new_unchecked((api().addNodeRef)(self.handle.as_ptr())) },
        }
    }
}

impl Drop for NodeRef {
    fn drop(&mut self) {
        // Safety: `self.handle` is a valid pointer
        unsafe { (api().freeNode)(self.handle.as_ptr()) }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum FilterMode {
    Parallel,
    ParallelRequests,
    Unordered,
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

pub trait Filter {
    type InstanceData;

    fn get_frame(&self) -> ffi::VSFilterGetFrame;
    fn free(&self) -> ffi::VSFilterFree;
    fn filter_mode(&self) -> FilterMode;
    fn instance_data(&mut self) -> &mut Self::InstanceData;
}
