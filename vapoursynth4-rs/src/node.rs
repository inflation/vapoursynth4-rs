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
    api::Api,
    core::Core,
    ffi,
    frame::{Frame, FrameContext, FrameType, FrameTypeAudio, FrameTypeVideo, VideoFrame},
    node::internal::FilterExtern,
    AudioInfo, VideoInfo,
};

pub use dependency::*;
pub use filter::*;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Node<T: FrameType> {
    handle: NonNull<ffi::VSNode>,
    api: Api,
    frame: std::marker::PhantomData<T>,
}

impl<T: FrameType> Node<T> {
    #[must_use]
    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::VSNode {
        self.handle.as_ptr()
    }

    #[must_use]
    #[inline]
    pub(crate) unsafe fn from_ptr(ptr: *mut ffi::VSNode, api: Api) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr),
            api,
            frame: std::marker::PhantomData,
        }
    }

    #[inline]
    #[must_use]
    pub fn set_linear_filter(&self) -> i32 {
        unsafe { (self.api.setLinearFilter)(self.as_ptr()) }
    }

    #[inline]
    pub fn set_cache_mode(&self, mode: CacheMode) {
        unsafe {
            (self.api.setCacheMode)(self.as_ptr(), mode);
        }
    }

    #[inline]
    pub fn set_cache_options(&self, fixed_size: i32, max_size: i32, max_history_size: i32) {
        unsafe {
            (self.api.setCacheOptions)(self.as_ptr(), fixed_size, max_size, max_history_size);
        }
    }

    /// # Errors
    ///
    /// Return the internal error message if the frame is not ready.
    pub fn get_frame(&self, n: i32) -> Result<Frame<T>, CString> {
        const LEN: i32 = 1024 * 10;
        let mut buf = vec![0; LEN as _];
        let ptr = unsafe { (self.api.getFrame)(n, self.as_ptr(), buf.as_mut_ptr(), LEN) };

        if ptr.is_null() {
            let mut buf = std::mem::ManuallyDrop::new(buf);
            Err(unsafe { CStr::from_ptr(buf.as_mut_ptr()).into() })
        } else {
            unsafe { Ok(Frame::<T>::from_ptr(ptr, self.api)) }
        }
    }

    // TODO: get_frame_async

    #[must_use]
    #[inline]
    pub fn get_frame_filter(&self, n: i32, ctx: &FrameContext) -> VideoFrame {
        unsafe {
            VideoFrame::from_ptr(
                (self.api.getFrameFilter)(n, self.as_ptr(), ctx.as_ptr()),
                self.api,
            )
        }
    }

    #[inline]
    pub fn request_frame_filter(&self, n: i32, ctx: &FrameContext) {
        unsafe {
            (self.api.requestFrameFilter)(n, self.as_ptr(), ctx.as_ptr());
        }
    }

    #[inline]
    pub fn release_frame_early(&self, n: i32, ctx: &FrameContext) {
        unsafe {
            (self.api.releaseFrameEarly)(self.as_ptr(), n, ctx.as_ptr());
        }
    }
}

impl<T: FrameType> Clone for Node<T> {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((self.api.addNodeRef)(self.as_ptr()), self.api) }
    }
}

impl<T: FrameType> Drop for Node<T> {
    fn drop(&mut self) {
        unsafe { (self.api.freeNode)(self.as_ptr()) }
    }
}

impl Node<FrameTypeVideo> {
    #[must_use]
    #[inline]
    pub fn info(&self) -> &VideoInfo {
        // SAFETY: `vi` is valid if the node is a video node
        unsafe { &*(self.api.getVideoInfo)(self.as_ptr()) }
    }

    /// # Panics
    ///
    /// Panics if the the dependency index is larger than [`i32::MAX`].
    pub fn new<F: Filter>(
        name: &str,
        info: &VideoInfo,
        filter: F,
        dependencies: &[ffi::VSFilterDependency],
        core: impl AsRef<Core>,
    ) -> Option<Self> {
        let filter = Box::new(filter);
        let name = CString::new(name).ok()?;
        let core = core.as_ref();
        let ptr = unsafe {
            (core.api().createVideoFilter2)(
                name.as_ptr(),
                info,
                F::filter_get_frame,
                Some(F::filter_free),
                F::FILTER_MODE,
                dependencies.as_ptr(),
                dependencies.len().try_into().unwrap(),
                Box::into_raw(filter).cast(),
                core.as_ptr(),
            )
        };

        NonNull::new(ptr).map(|handle| Self {
            handle,
            api: core.api(),
            frame: std::marker::PhantomData,
        })
    }
}

impl Node<FrameTypeAudio> {
    #[must_use]
    #[inline]
    pub fn info(&self) -> &AudioInfo {
        // SAFETY: `vi` is valid if the node is a video node
        unsafe { &*(self.api.getAudioInfo)(self.as_ptr()) }
    }

    /// # Panics
    ///
    /// Panics if the the dependency index is larger than [`i32::MAX`].
    pub fn new<F: Filter>(
        name: &str,
        info: &AudioInfo,
        filter: F,
        dependencies: &[ffi::VSFilterDependency],
        core: impl AsRef<Core>,
    ) -> Option<Self> {
        let filter = Box::new(filter);
        let name = CString::new(name).ok()?;
        let core = core.as_ref();
        let ptr = unsafe {
            (core.api().createAudioFilter2)(
                name.as_ptr(),
                info,
                F::filter_get_frame,
                Some(F::filter_free),
                F::FILTER_MODE,
                dependencies.as_ptr(),
                dependencies.len().try_into().unwrap(),
                Box::into_raw(filter).cast(),
                core.as_ptr(),
            )
        };
        NonNull::new(ptr).map(|handle| Self {
            handle,
            api: core.api(),
            frame: std::marker::PhantomData,
        })
    }
}

pub type FilterMode = ffi::VSFilterMode;
pub type CacheMode = ffi::VSCacheMode;

pub type VideoNode = Node<FrameTypeVideo>;
pub type AudioNode = Node<FrameTypeVideo>;
