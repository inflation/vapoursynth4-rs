/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

mod dependency;
mod filter;
pub(crate) mod internal;

use std::{
    ffi::{CStr, CString},
    ptr::NonNull,
};

use crate::{
    api,
    core::Core,
    ffi,
    frame::{internal::FrameFromPtr, AudioFrame, Frame, FrameContext, VideoFrame},
    node::internal::FilterExtern,
    AudioInfo, VideoInfo,
};

pub use dependency::*;
pub use filter::*;

pub trait Node: Sized + crate::_private::Sealed {
    type FrameType: Frame;

    #[must_use]
    fn as_ptr(&self) -> *const ffi::VSNode;

    #[must_use]
    fn as_mut_ptr(&mut self) -> *mut ffi::VSNode {
        self.as_ptr().cast_mut()
    }

    #[must_use]
    fn get_frame_filter(&self, n: i32, ctx: &mut FrameContext) -> Self::FrameType;

    fn set_linear_filter(&mut self) -> i32 {
        unsafe { (api().setLinearFilter)(self.as_mut_ptr()) }
    }

    fn set_cache_mode(&mut self, mode: CacheMode) {
        unsafe {
            (api().setCacheMode)(self.as_mut_ptr(), mode);
        }
    }

    fn set_cache_options(&mut self, fixed_size: i32, max_size: i32, max_history_size: i32) {
        unsafe {
            (api().setCacheOptions)(self.as_mut_ptr(), fixed_size, max_size, max_history_size);
        }
    }

    /// # Errors
    ///
    /// Return the internal error message if the frame is not ready.
    fn get_frame(&self, n: i32) -> Result<Self::FrameType, CString> {
        let mut buf = vec![0; 1024];
        let ptr = unsafe { (api().getFrame)(n, self.as_ptr().cast_mut(), buf.as_mut_ptr(), 1024) };

        if ptr.is_null() {
            let mut buf = std::mem::ManuallyDrop::new(buf);
            Err(unsafe { CStr::from_ptr(buf.as_mut_ptr()).into() })
        } else {
            unsafe { Ok(Self::FrameType::from_ptr(ptr)) }
        }
    }

    // TODO: Find a better way to handle callbacks
    fn get_frame_async<D, F, Fr>(&self, _n: i32, _data: &mut D)
    where
        F: Fn(D, Fr, i32) -> Result<(), String>,
        Fr: Frame,
    {
        todo!()
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct VideoNode {
    handle: NonNull<ffi::VSNode>,
}

impl crate::_private::Sealed for VideoNode {}
impl Node for VideoNode {
    type FrameType = VideoFrame;

    #[must_use]
    fn as_ptr(&self) -> *const ffi::VSNode {
        self.handle.as_ptr()
    }

    #[must_use]
    fn get_frame_filter(&self, n: i32, ctx: &mut FrameContext) -> Self::FrameType {
        unsafe {
            VideoFrame::from_ptr((api().getFrameFilter)(
                n,
                self.as_ptr().cast_mut(),
                ctx.as_mut_ptr(),
            ))
        }
    }
}

impl VideoNode {
    #[must_use]
    pub(crate) unsafe fn from_ptr(ptr: *mut ffi::VSNode) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr),
        }
    }

    #[must_use]
    pub fn get_info(&self) -> *const VideoInfo {
        // safety: `vi` is valid if the node is a video node
        unsafe { (api().getVideoInfo)(self.as_ptr().cast_mut()) }
    }

    /// # Panics
    ///
    /// Panics if the the dependency index is larger than [`i32::MAX`].
    pub fn new<F: Filter>(
        name: &str,
        info: &VideoInfo,
        filter: F,
        dependencies: &[ffi::VSFilterDependency],
        core: &mut Core,
    ) -> Option<Self> {
        let filter = Box::new(filter);
        let name = CString::new(name).ok()?;
        let ptr = unsafe {
            (api().createVideoFilter2)(
                name.as_ptr(),
                info,
                F::filter_get_frame,
                Some(F::filter_free),
                F::FILTER_MODE,
                dependencies.as_ptr(),
                dependencies.len().try_into().unwrap(),
                Box::into_raw(filter).cast(),
                core.as_mut_ptr(),
            )
        };
        NonNull::new(ptr).map(|handle| Self { handle })
    }
}

impl Clone for VideoNode {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((api().addNodeRef)(self.as_ptr().cast_mut())) }
    }
}

impl Drop for VideoNode {
    fn drop(&mut self) {
        unsafe { (api().freeNode)(self.as_mut_ptr()) }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct AudioNode {
    handle: NonNull<ffi::VSNode>,
}

impl crate::_private::Sealed for AudioNode {}
impl Node for AudioNode {
    type FrameType = AudioFrame;
    #[must_use]
    fn as_ptr(&self) -> *const ffi::VSNode {
        self.handle.as_ptr()
    }

    fn get_frame_filter(&self, n: i32, ctx: &mut FrameContext) -> Self::FrameType {
        unsafe {
            AudioFrame::from_ptr((api().getFrameFilter)(
                n,
                self.as_ptr().cast_mut(),
                ctx.as_mut_ptr(),
            ))
        }
    }
}

impl AudioNode {
    #[must_use]
    pub(crate) unsafe fn from_ptr(ptr: *mut ffi::VSNode) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr),
        }
    }

    #[must_use]
    pub fn get_info(&self) -> *const AudioInfo {
        // safety: `ai` is valid if the node is an audio node
        unsafe { (api().getAudioInfo)(self.as_ptr().cast_mut()) }
    }

    /// # Panics
    ///
    /// Panics if the the dependency index is larger than [`i32::MAX`].
    pub fn new<F: Filter>(
        name: &str,
        info: &AudioInfo,
        filter: F,
        dependencies: &[ffi::VSFilterDependency],
        core: &mut Core,
    ) -> Option<Self> {
        let filter = Box::new(filter);
        let name = CString::new(name).ok()?;
        let ptr = unsafe {
            (api().createAudioFilter2)(
                name.as_ptr(),
                info,
                F::filter_get_frame,
                Some(F::filter_free),
                F::FILTER_MODE,
                dependencies.as_ptr(),
                dependencies.len().try_into().unwrap(),
                Box::into_raw(filter).cast(),
                core.as_mut_ptr(),
            )
        };
        NonNull::new(ptr).map(|handle| Self { handle })
    }
}

impl Clone for AudioNode {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((api().addNodeRef)(self.as_ptr().cast_mut())) }
    }
}

impl Drop for AudioNode {
    fn drop(&mut self) {
        unsafe { (api().freeNode)(self.as_mut_ptr()) }
    }
}

pub type FilterMode = ffi::VSFilterMode;
pub type CacheMode = ffi::VSCacheMode;
